import { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { TabulationService } from '../services/tabulation';
import { AttendanceService } from '../services/attendance';
import type { 
  MatchSeries, 
  MatchResponse, 
  Event,
  AllocationRole,
  TwoTeamSpeakerRole,
  CreateAllocationRequest,
} from '../services/types';

interface AvailableUser {
  user_id: string;
  username: string;
  is_checked_in: boolean;
  checked_in_at: string | null;
  current_allocations: {
    match_id: string;
    room_name: string | null;
    role: AllocationRole;
  }[];
}

export default function AllocationPage() {
  const { eventId, seriesId } = useParams<{ eventId: string; seriesId: string }>();

  const [event, setEvent] = useState<Event | null>(null);
  const [series, setSeries] = useState<MatchSeries | null>(null);
  const [matches, setMatches] = useState<MatchResponse[]>([]);
  const [availableUsers, setAvailableUsers] = useState<AvailableUser[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Filter states
  const [userFilter, setUserFilter] = useState<'all' | 'checked-in' | 'not-checked-in'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  
  // Selection states
  const [selectedUser, setSelectedUser] = useState<AvailableUser | null>(null);
  const [selectedMatch, setSelectedMatch] = useState<string | null>(null);
  const [selectedTeam, setSelectedTeam] = useState<string | null>(null);
  const [selectedRole, setSelectedRole] = useState<AllocationRole>('speaker');
  const [speakerRole, setSpeakerRole] = useState<TwoTeamSpeakerRole>('prime_minister');
  const [isChair, setIsChair] = useState(false);

  // Suppress unused variable warnings
  void series;

  const loadData = useCallback(async () => {
    if (!eventId || !seriesId) return;

    setIsLoading(true);
    setError(null);

    try {
      // Load event details
      const eventData = await AttendanceService.getEvent(eventId);
      setEvent(eventData);

      // Load series details
      const seriesResponse = await TabulationService.listSeries(eventId);
      const currentSeries = seriesResponse.series.find(s => s.id === seriesId);
      setSeries(currentSeries || null);

      // Load matches for this series
      const matchesResponse = await TabulationService.listMatches({ seriesId });
      setMatches(matchesResponse.matches);

      // Load available users - combination of checked-in users and all users
      await loadAvailableUsers(eventId, matchesResponse.matches);

    } catch (err) {
      setError('Failed to load data. Please try again.');
      console.error('Failed to load data:', err);
    } finally {
      setIsLoading(false);
    }
  }, [eventId, seriesId]);

  const loadAvailableUsers = async (eventId: string, currentMatches: MatchResponse[]) => {
    try {
      // Get checked-in users from attendance
      const checkedInResponse = await AttendanceService.getEventAttendance(eventId);
      
      // Build allocations map from matches
      const userAllocations = new Map<string, { match_id: string; room_name: string | null; role: AllocationRole }[]>();
      
      for (const match of currentMatches) {
        // Speakers (only track registered users, not guests)
        for (const team of match.teams) {
          for (const speaker of team.speakers) {
            if (speaker.user_id) {
              const existing = userAllocations.get(speaker.user_id) || [];
              existing.push({
                match_id: match.id,
                room_name: match.room_name,
                role: 'speaker',
              });
              userAllocations.set(speaker.user_id, existing);
            }
          }
        }
        // Adjudicators (only track registered users, not guests)
        for (const adj of match.adjudicators) {
          if (adj.user_id) {
            const existing = userAllocations.get(adj.user_id) || [];
            existing.push({
              match_id: match.id,
              room_name: match.room_name,
              role: adj.is_voting ? 'voting_adjudicator' : 'non_voting_adjudicator',
            });
            userAllocations.set(adj.user_id, existing);
          }
        }
      }

      // Build available users from checked-in attendance records
      const users: AvailableUser[] = (checkedInResponse.attendance || []).map((record) => ({
        user_id: record.user_id,
        username: record.username,
        is_checked_in: true,
        checked_in_at: record.checked_in_at || null,
        current_allocations: userAllocations.get(record.user_id) || [],
      }));

      // Sort alphabetically
      users.sort((a, b) => a.username.localeCompare(b.username));

      setAvailableUsers(users);
    } catch (err) {
      console.error('Failed to load users:', err);
      // Continue without users list - will show error in UI
    }
  };

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Auto-dismiss success messages
  useEffect(() => {
    if (successMessage) {
      const timer = setTimeout(() => setSuccessMessage(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [successMessage]);

  const handleAllocate = async () => {
    if (!selectedUser || !selectedMatch) {
      setError('Please select a user and a match');
      return;
    }

    try {
      const data: CreateAllocationRequest = {
        match_id: selectedMatch,
        user_id: selectedUser.user_id,
        role: selectedRole,
      };

      if (selectedRole === 'speaker' && selectedTeam) {
        data.team_id = selectedTeam;
        data.two_team_speaker_role = speakerRole;
      }

      if (selectedRole === 'voting_adjudicator' || selectedRole === 'non_voting_adjudicator') {
        data.is_chair = isChair;
      }

      await TabulationService.createAllocation(data);
      setSuccessMessage(`${selectedUser.username} allocated successfully!`);
      
      // Reset selection
      setSelectedUser(null);
      setSelectedTeam(null);
      
      // Reload data
      await loadData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to allocate user');
    }
  };

  const handleRemoveAllocation = async (allocationId: string) => {
    try {
      await TabulationService.deleteAllocation(allocationId);
      setSuccessMessage('Allocation removed');
      await loadData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to remove allocation');
    }
  };

  const filteredUsers = availableUsers.filter(user => {
    // Apply search filter
    if (searchQuery && !user.username.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false;
    }
    // Apply check-in filter
    if (userFilter === 'checked-in' && !user.is_checked_in) {
      return false;
    }
    if (userFilter === 'not-checked-in' && user.is_checked_in) {
      return false;
    }
    return true;
  });

  const getRoleLabel = (role: AllocationRole) => {
    const labels: Record<AllocationRole, string> = {
      speaker: 'Speaker',
      voting_adjudicator: 'Voting Adjudicator',
      non_voting_adjudicator: 'Shadow Judge',
      resource: 'Resource',
    };
    return labels[role] || role;
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <svg className="animate-spin h-12 w-12 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <nav className="flex mb-4" aria-label="Breadcrumb">
            <ol className="flex items-center space-x-2 text-sm text-gray-500">
              <li><Link to="/events" className="hover:text-gray-700">Events</Link></li>
              <li><span className="mx-2">/</span></li>
              <li><Link to={`/events/${eventId}`} className="hover:text-gray-700">{event?.title}</Link></li>
              <li><span className="mx-2">/</span></li>
              <li><Link to={`/admin/events/${eventId}/matches`} className="hover:text-gray-700">Matches</Link></li>
              <li><span className="mx-2">/</span></li>
              <li className="text-gray-900 font-medium">Allocate</li>
            </ol>
          </nav>

          <div className="flex justify-between items-start">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Allocate People</h1>
              <p className="mt-2 text-sm text-gray-600">
                Assign speakers and adjudicators to matches for {event?.title}
              </p>
            </div>
            <Link
              to={`/admin/events/${eventId}/matches`}
              className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
            >
              ← Back to Matches
            </Link>
          </div>
        </div>

        {/* Messages */}
        {error && (
          <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
            <div className="flex justify-between">
              <p className="text-sm text-red-600">{error}</p>
              <button onClick={() => setError(null)} className="text-red-400 hover:text-red-600">×</button>
            </div>
          </div>
        )}
        {successMessage && (
          <div className="mb-4 bg-green-50 border border-green-200 rounded-md p-4">
            <p className="text-sm text-green-600">{successMessage}</p>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Users Panel */}
          <div className="lg:col-span-1">
            <div className="bg-white rounded-lg shadow-sm">
              <div className="px-4 py-3 border-b border-gray-200">
                <h2 className="text-lg font-medium text-gray-900">Available People</h2>
                <p className="text-sm text-gray-500 mt-1">
                  {availableUsers.filter(u => u.is_checked_in).length} checked in • {availableUsers.length} total
                </p>
              </div>
              
              {/* Filters */}
              <div className="p-4 border-b border-gray-200 space-y-3">
                <input
                  type="text"
                  placeholder="Search by name..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
                <div className="flex space-x-2">
                  <button
                    onClick={() => setUserFilter('all')}
                    className={`px-3 py-1 text-xs rounded-full ${
                      userFilter === 'all' ? 'bg-indigo-100 text-indigo-800' : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    All
                  </button>
                  <button
                    onClick={() => setUserFilter('checked-in')}
                    className={`px-3 py-1 text-xs rounded-full ${
                      userFilter === 'checked-in' ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    ✓ Checked In
                  </button>
                  <button
                    onClick={() => setUserFilter('not-checked-in')}
                    className={`px-3 py-1 text-xs rounded-full ${
                      userFilter === 'not-checked-in' ? 'bg-amber-100 text-amber-800' : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    Not Checked In
                  </button>
                </div>
              </div>

              {/* Users List */}
              <div className="divide-y divide-gray-100 max-h-[500px] overflow-y-auto">
                {filteredUsers.length === 0 ? (
                  <div className="p-4 text-center text-gray-500 text-sm">
                    No users found
                  </div>
                ) : (
                  filteredUsers.map((user) => (
                    <div
                      key={user.user_id}
                      className={`p-3 cursor-pointer hover:bg-gray-50 ${
                        selectedUser?.user_id === user.user_id ? 'bg-indigo-50 border-l-4 border-indigo-600' : ''
                      }`}
                      onClick={() => setSelectedUser(user)}
                    >
                      <div className="flex items-center justify-between">
                        <div>
                          <span className={`font-medium ${user.is_checked_in ? 'text-gray-900' : 'text-gray-500'}`}>
                            {user.username}
                          </span>
                          {user.is_checked_in ? (
                            <span className="ml-2 text-xs text-green-600">✓</span>
                          ) : (
                            <span className="ml-2 text-xs text-amber-600">(not checked in)</span>
                          )}
                        </div>
                        {user.current_allocations.length > 0 && (
                          <span className="text-xs bg-gray-100 text-gray-600 px-2 py-0.5 rounded">
                            {user.current_allocations.length} allocated
                          </span>
                        )}
                      </div>
                      {user.current_allocations.length > 0 && (
                        <div className="mt-1 text-xs text-gray-500">
                          {user.current_allocations.map((alloc, i) => (
                            <span key={i}>
                              {alloc.room_name || 'Match'} ({getRoleLabel(alloc.role)})
                              {i < user.current_allocations.length - 1 ? ', ' : ''}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>

          {/* Allocation Panel */}
          <div className="lg:col-span-2">
            {/* Allocation Form */}
            {selectedUser && (
              <div className="bg-white rounded-lg shadow-sm mb-6 p-4">
                <h3 className="font-medium text-gray-900 mb-4">
                  Allocate: <span className="text-indigo-600">{selectedUser.username}</span>
                  {!selectedUser.is_checked_in && (
                    <span className="ml-2 text-xs text-amber-600 font-normal">(not checked in)</span>
                  )}
                </h3>

                <div className="grid grid-cols-2 gap-4">
                  {/* Match Selection */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Match</label>
                    <select
                      value={selectedMatch || ''}
                      onChange={(e) => {
                        setSelectedMatch(e.target.value);
                        setSelectedTeam(null);
                      }}
                      className="w-full border border-gray-300 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    >
                      <option value="">Select a match...</option>
                      {matches.map((match, index) => (
                        <option key={match.id} value={match.id}>
                          {match.room_name || `Match ${index + 1}`}
                        </option>
                      ))}
                    </select>
                  </div>

                  {/* Role Selection */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Role</label>
                    <select
                      value={selectedRole}
                      onChange={(e) => setSelectedRole(e.target.value as AllocationRole)}
                      className="w-full border border-gray-300 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    >
                      <option value="speaker">Speaker</option>
                      <option value="voting_adjudicator">Voting Adjudicator</option>
                      <option value="non_voting_adjudicator">Shadow Judge</option>
                      <option value="resource">Resource</option>
                    </select>
                  </div>

                  {/* Team Selection (for speakers) */}
                  {selectedRole === 'speaker' && selectedMatch && (
                    <>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">Team</label>
                        <select
                          value={selectedTeam || ''}
                          onChange={(e) => setSelectedTeam(e.target.value)}
                          className="w-full border border-gray-300 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        >
                          <option value="">Select a team...</option>
                          {matches.find(m => m.id === selectedMatch)?.teams.map((team) => (
                            <option key={team.id} value={team.id}>
                              {team.two_team_position === 'government' ? '🏛️ Government' :
                               team.two_team_position === 'opposition' ? '⚔️ Opposition' :
                               team.four_team_position?.replace('_', ' ') || 'Team'}
                            </option>
                          ))}
                        </select>
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">Speaker Position</label>
                        <select
                          value={speakerRole}
                          onChange={(e) => setSpeakerRole(e.target.value as TwoTeamSpeakerRole)}
                          className="w-full border border-gray-300 rounded-md py-2 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        >
                          <option value="first_speaker">First Speaker</option>
                          <option value="second_speaker">Second Speaker</option>
                          <option value="third_speaker">Third Speaker</option>
                          <option value="reply_speaker">Reply Speaker</option>
                        </select>
                      </div>
                    </>
                  )}

                  {/* Chair option (for adjudicators) */}
                  {(selectedRole === 'voting_adjudicator' || selectedRole === 'non_voting_adjudicator') && (
                    <div className="col-span-2">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={isChair}
                          onChange={(e) => setIsChair(e.target.checked)}
                          className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                        />
                        <span className="ml-2 text-sm text-gray-700">Chair Adjudicator</span>
                      </label>
                    </div>
                  )}
                </div>

                <div className="mt-4 flex justify-end space-x-3">
                  <button
                    onClick={() => setSelectedUser(null)}
                    className="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleAllocate}
                    disabled={!selectedMatch || (selectedRole === 'speaker' && !selectedTeam)}
                    className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
                  >
                    Allocate
                  </button>
                </div>
              </div>
            )}

            {/* Matches Overview */}
            <div className="space-y-4">
              {matches.map((match, index) => (
                <div key={match.id} className="bg-white rounded-lg shadow-sm">
                  <div className="px-4 py-3 border-b border-gray-200">
                    <h3 className="font-medium text-gray-900">
                      {match.room_name || `Match ${index + 1}`}
                    </h3>
                    {match.motion && (
                      <p className="text-sm text-gray-500 mt-1 italic">"{match.motion}"</p>
                    )}
                  </div>

                  <div className="p-4">
                    {/* Teams */}
                    <div className="grid grid-cols-2 gap-4 mb-4">
                      {match.teams.map((team) => (
                        <div key={team.id} className="bg-gray-50 rounded-lg p-3">
                          <h4 className="text-sm font-medium text-gray-700 mb-2">
                            {team.two_team_position === 'government' ? '🏛️ Government' :
                             team.two_team_position === 'opposition' ? '⚔️ Opposition' :
                             team.four_team_position?.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase()) || 'Team'}
                          </h4>
                          {team.speakers.length > 0 ? (
                            <ul className="space-y-1">
                              {team.speakers.map((speaker) => (
                                <li key={speaker.allocation_id} className="flex items-center justify-between text-sm">
                                  <span className={speaker.was_checked_in ? '' : 'text-gray-400 italic'}>
                                    {speaker.username}
                                    {speaker.two_team_speaker_role && (
                                      <span className="text-gray-400 ml-1">
                                        ({speaker.two_team_speaker_role.replace('_', ' ')})
                                      </span>
                                    )}
                                    {!speaker.was_checked_in && (
                                      <span className="ml-1 text-xs text-amber-600">(not checked in)</span>
                                    )}
                                  </span>
                                  <button
                                    onClick={() => handleRemoveAllocation(speaker.allocation_id)}
                                    className="text-red-400 hover:text-red-600 ml-2"
                                    title="Remove"
                                  >
                                    ×
                                  </button>
                                </li>
                              ))}
                            </ul>
                          ) : (
                            <p className="text-sm text-gray-400 italic">No speakers assigned</p>
                          )}
                        </div>
                      ))}
                    </div>

                    {/* Adjudicators */}
                    <div className="border-t border-gray-100 pt-3">
                      <h4 className="text-sm font-medium text-gray-700 mb-2">⚖️ Adjudicators</h4>
                      {match.adjudicators.length > 0 ? (
                        <ul className="space-y-1">
                          {match.adjudicators.map((adj) => (
                            <li key={adj.allocation_id} className="flex items-center justify-between text-sm">
                              <span className={adj.was_checked_in ? '' : 'text-gray-400 italic'}>
                                {adj.username}
                                {adj.is_chair && <span className="ml-1 text-indigo-600">(Chair)</span>}
                                {!adj.is_voting && <span className="ml-1 text-gray-400">(Shadow)</span>}
                                {!adj.was_checked_in && (
                                  <span className="ml-1 text-xs text-amber-600">(not checked in)</span>
                                )}
                              </span>
                              <button
                                onClick={() => handleRemoveAllocation(adj.allocation_id)}
                                className="text-red-400 hover:text-red-600 ml-2"
                                title="Remove"
                              >
                                ×
                              </button>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p className="text-sm text-gray-400 italic">No adjudicators assigned</p>
                      )}
                    </div>
                  </div>
                </div>
              ))}

              {matches.length === 0 && (
                <div className="bg-white rounded-lg shadow-sm p-8 text-center">
                  <p className="text-gray-500">No matches created yet. Create matches first before allocating.</p>
                  <Link
                    to={`/admin/events/${eventId}/matches`}
                    className="mt-4 inline-flex items-center text-indigo-600 hover:text-indigo-800"
                  >
                    ← Go to Matches
                  </Link>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
