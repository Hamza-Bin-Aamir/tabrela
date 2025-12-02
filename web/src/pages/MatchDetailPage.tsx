import { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { TabulationService } from '../services/tabulation';
import type {
  MatchResponse,
  MatchSeries,
  AllocationPoolResponse,
  CheckedInUserInfo,
  MatchTeam,
  AdjudicatorInfo,
  BallotResponse,
  FourTeamPosition,
  TwoTeamPosition,
  TwoTeamSpeakerRole,
  FourTeamSpeakerRole,
  AllocationRole,
} from '../services/types';

type DragItem = CheckedInUserInfo | { guest_name: string; isGuest: true };

export default function MatchDetailPage() {
  const { matchId } = useParams<{ matchId: string }>();
  
  const [match, setMatch] = useState<MatchResponse | null>(null);
  const [series, setSeries] = useState<MatchSeries | null>(null);
  const [allocationPool, setAllocationPool] = useState<AllocationPoolResponse | null>(null);
  const [ballots, setBallots] = useState<BallotResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [draggedUser, setDraggedUser] = useState<DragItem | null>(null);
  const [activeTab, setActiveTab] = useState<'allocations' | 'results'>('allocations');
  const [showGuestModal, setShowGuestModal] = useState(false);
  const [guestName, setGuestName] = useState('');
  const [guestRole, setGuestRole] = useState<AllocationRole>('speaker');
  const [guestTeamId, setGuestTeamId] = useState<string>('');
  const [guestSpeakerRole, setGuestSpeakerRole] = useState<TwoTeamSpeakerRole | FourTeamSpeakerRole | ''>('');

  const loadMatchData = useCallback(async () => {
    if (!matchId) return;
    
    try {
      setLoading(true);
      // Get match details first
      const matchData = await TabulationService.getMatch(matchId);
      setMatch(matchData);
      
      // Get series to get the allocation pool
      const seriesData = await TabulationService.getSeries(matchData.series_id);
      setSeries(seriesData);
      
      // Get allocation pool for the series
      const poolData = await TabulationService.getAllocationPool(matchData.series_id);
      setAllocationPool(poolData);

      // Try to load ballots for admin
      try {
        const ballotsData = await TabulationService.getMatchBallots(matchId);
        setBallots(ballotsData);
      } catch {
        // May not have admin access or no ballots yet
        setBallots([]);
      }

      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load match data');
    } finally {
      setLoading(false);
    }
  }, [matchId]);

  useEffect(() => {
    loadMatchData();
  }, [loadMatchData]);

  const handleDragStart = (user: DragItem) => {
    setDraggedUser(user);
  };

  const handleDragEnd = () => {
    setDraggedUser(null);
  };

  const handleDropSpeaker = async (
    teamId: string,
    twoTeamRole?: TwoTeamSpeakerRole,
    fourTeamRole?: FourTeamSpeakerRole
  ) => {
    if (!draggedUser || !matchId) return;

    try {
      const isGuest = 'isGuest' in draggedUser && draggedUser.isGuest;
      await TabulationService.createAllocation({
        match_id: matchId,
        user_id: isGuest ? undefined : (draggedUser as CheckedInUserInfo).user_id,
        guest_name: isGuest ? draggedUser.guest_name : undefined,
        role: 'speaker',
        team_id: teamId,
        two_team_speaker_role: twoTeamRole,
        four_team_speaker_role: fourTeamRole,
      });
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to allocate user');
    }
  };

  const handleDropAdjudicator = async (isVoting: boolean, isChair: boolean = false) => {
    if (!draggedUser || !matchId) return;

    try {
      const isGuest = 'isGuest' in draggedUser && draggedUser.isGuest;
      await TabulationService.createAllocation({
        match_id: matchId,
        user_id: isGuest ? undefined : (draggedUser as CheckedInUserInfo).user_id,
        guest_name: isGuest ? draggedUser.guest_name : undefined,
        role: isVoting ? 'voting_adjudicator' : 'non_voting_adjudicator',
        is_chair: isChair,
      });
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to allocate user');
    }
  };

  const handleDropResource = async () => {
    if (!draggedUser || !matchId) return;

    try {
      const isGuest = 'isGuest' in draggedUser && draggedUser.isGuest;
      await TabulationService.createAllocation({
        match_id: matchId,
        user_id: isGuest ? undefined : (draggedUser as CheckedInUserInfo).user_id,
        guest_name: isGuest ? draggedUser.guest_name : undefined,
        role: 'resource',
      });
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to allocate user');
    }
  };

  const handleAddGuest = async () => {
    if (!guestName.trim() || !matchId) return;

    try {
      const allocationData: Parameters<typeof TabulationService.createAllocation>[0] = {
        match_id: matchId,
        guest_name: guestName.trim(),
        role: guestRole,
      };

      if (guestRole === 'speaker') {
        if (!guestTeamId) {
          setError('Please select a team for the speaker');
          return;
        }
        allocationData.team_id = guestTeamId;
        if (guestSpeakerRole) {
          if (series?.team_format === 'two_team') {
            allocationData.two_team_speaker_role = guestSpeakerRole as TwoTeamSpeakerRole;
          } else {
            allocationData.four_team_speaker_role = guestSpeakerRole as FourTeamSpeakerRole;
          }
        }
      } else if (guestRole === 'voting_adjudicator' || guestRole === 'non_voting_adjudicator') {
        // Adjudicator - no team needed
      }

      await TabulationService.createAllocation(allocationData);
      setShowGuestModal(false);
      setGuestName('');
      setGuestRole('speaker');
      setGuestTeamId('');
      setGuestSpeakerRole('');
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add guest');
    }
  };

  const handleRemoveAllocation = async (allocationId: string) => {
    try {
      await TabulationService.deleteAllocation(allocationId);
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove allocation');
    }
  };

  const handleToggleRelease = async (field: 'scores_released' | 'rankings_released') => {
    if (!matchId || !match) return;

    try {
      const newValue = !(match[field]);
      await TabulationService.toggleRelease(matchId, {
        [field]: newValue,
      });
      await loadMatchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update score release');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error || !match) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
          {error || 'Match not found'}
        </div>
      </div>
    );
  }

  // Get all users from allocation pool - allow multiple allocations for friendly matches
  const availableUsers = allocationPool?.checked_in_users || [];

  const isFourTeam = series?.team_format === 'four_team';

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between">
            <div>
              <Link 
                to={`/admin/events/${series?.event_id}/matches`}
                className="text-sm text-gray-500 hover:text-gray-700"
              >
                ← Back to Series
              </Link>
              <h1 className="text-2xl font-bold text-gray-900 mt-1">
                {match.series_name}
              </h1>
              <p className="text-sm text-gray-500">
                {isFourTeam ? '4-Team British Parliamentary' : '2-Team'} • Room: {match.room_name || 'TBA'}
              </p>
            </div>
            <div className="flex items-center gap-4">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                match.status === 'completed' ? 'bg-green-100 text-green-800' :
                match.status === 'in_progress' ? 'bg-blue-100 text-blue-800' :
                match.status === 'published' ? 'bg-purple-100 text-purple-800' :
                'bg-gray-100 text-gray-800'
              }`}>
                {match.status.replace('_', ' ')}
              </span>
            </div>
          </div>
          
          {/* Tabs */}
          <div className="mt-4 border-b border-gray-200">
            <nav className="-mb-px flex space-x-8">
              <button
                onClick={() => setActiveTab('allocations')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'allocations'
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                Allocations
              </button>
              <button
                onClick={() => setActiveTab('results')}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === 'results'
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                Results & Scores
              </button>
            </nav>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
        {activeTab === 'allocations' ? (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Available Users Panel */}
            <div className="bg-white rounded-lg shadow p-4">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold flex items-center gap-2">
                  Available Participants
                  <span className="text-sm font-normal text-gray-500">
                    ({availableUsers.length})
                  </span>
                </h2>
                <button
                  onClick={() => setShowGuestModal(true)}
                  className="text-sm px-3 py-1 bg-indigo-600 text-white rounded hover:bg-indigo-700"
                >
                  + Add Guest
                </button>
              </div>
              
              <div className="space-y-2 max-h-[calc(100vh-300px)] overflow-y-auto">
                {availableUsers.length === 0 ? (
                  <p className="text-gray-500 text-sm text-center py-4">
                    No available participants
                  </p>
                ) : (
                  availableUsers.map(user => (
                    <div
                      key={user.user_id}
                      draggable
                      onDragStart={() => handleDragStart(user)}
                      onDragEnd={handleDragEnd}
                      className={`p-3 rounded-lg cursor-grab hover:bg-gray-100 transition-colors border ${
                        user.is_allocated 
                          ? 'bg-yellow-50 border-yellow-300' 
                          : 'bg-gray-50 border-gray-200'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <div className="font-medium text-sm">{user.username}</div>
                        {user.is_allocated && (
                          <span className="text-xs px-1.5 py-0.5 bg-yellow-200 text-yellow-800 rounded">
                            In use
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-gray-500">
                        {user.checked_in_at 
                          ? `Checked in at ${new Date(user.checked_in_at).toLocaleTimeString()}`
                          : 'Not checked in'}
                      </div>
                      {user.current_allocation && (
                        <div className="text-xs text-yellow-700 mt-1">
                          {user.current_allocation.role.replace('_', ' ')} in {user.current_allocation.room_name || 'another match'}
                        </div>
                      )}
                    </div>
                  ))
                )}
              </div>
            </div>

            {/* Allocation Zones */}
            <div className="lg:col-span-2 space-y-6">
              {/* Teams / Speaker Positions */}
              <div className="bg-white rounded-lg shadow p-4">
                <h2 className="text-lg font-semibold mb-4">
                  {isFourTeam ? 'Teams' : 'Speakers'}
                </h2>

                {isFourTeam ? (
                  <FourTeamLayout
                    teams={match.teams}
                    draggedUser={draggedUser}
                    onDrop={handleDropSpeaker}
                    onRemove={handleRemoveAllocation}
                  />
                ) : (
                  <TwoTeamLayout
                    teams={match.teams}
                    draggedUser={draggedUser}
                    onDrop={handleDropSpeaker}
                    onRemove={handleRemoveAllocation}
                  />
                )}
              </div>

              {/* Adjudicators */}
              <div className="bg-white rounded-lg shadow p-4">
                <h2 className="text-lg font-semibold mb-4">Adjudicators</h2>
                
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {/* Voting Adjudicators */}
                  <AdjudicatorDropZone
                    title="Voting Adjudicators"
                    adjudicators={match.adjudicators.filter(a => a.is_voting)}
                    draggedUser={draggedUser}
                    onDrop={() => handleDropAdjudicator(true)}
                    onRemove={handleRemoveAllocation}
                    className="border-green-200 bg-green-50"
                  />

                  {/* Non-Voting Adjudicators */}
                  <AdjudicatorDropZone
                    title="Non-Voting Adjudicators"
                    adjudicators={match.adjudicators.filter(a => !a.is_voting)}
                    draggedUser={draggedUser}
                    onDrop={() => handleDropAdjudicator(false)}
                    onRemove={handleRemoveAllocation}
                    className="border-yellow-200 bg-yellow-50"
                  />
                </div>
              </div>

              {/* Resources */}
              <div className="bg-white rounded-lg shadow p-4">
                <h2 className="text-lg font-semibold mb-4">Resources</h2>
                
                <ResourceDropZone
                  teams={match.teams}
                  draggedUser={draggedUser}
                  onDrop={handleDropResource}
                  onRemove={handleRemoveAllocation}
                />
              </div>
            </div>
          </div>
        ) : (
          <ResultsTab
            match={match}
            ballots={ballots}
            onToggleRelease={handleToggleRelease}
          />
        )}
      </main>

      {/* Add Guest Modal */}
      {showGuestModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold">Add Guest Participant</h3>
              <button
                onClick={() => setShowGuestModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                ✕
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Guest Name *
                </label>
                <input
                  type="text"
                  value={guestName}
                  onChange={(e) => setGuestName(e.target.value)}
                  placeholder="Enter guest name"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Role *
                </label>
                <select
                  value={guestRole}
                  onChange={(e) => setGuestRole(e.target.value as AllocationRole)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                >
                  <option value="speaker">Speaker</option>
                  <option value="voting_adjudicator">Voting Adjudicator</option>
                  <option value="non_voting_adjudicator">Non-Voting Adjudicator (Shadow)</option>
                </select>
              </div>

              {guestRole === 'speaker' && match && (
                <>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Team *
                    </label>
                    <select
                      value={guestTeamId}
                      onChange={(e) => setGuestTeamId(e.target.value)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    >
                      <option value="">Select a team</option>
                      {match.teams.map((team) => (
                        <option key={team.id} value={team.id}>
                          {team.four_team_position?.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase()) ||
                           team.two_team_position?.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase()) ||
                           team.team_name || 'Team'}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Speaker Position
                    </label>
                    <select
                      value={guestSpeakerRole}
                      onChange={(e) => setGuestSpeakerRole(e.target.value as TwoTeamSpeakerRole | FourTeamSpeakerRole)}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    >
                      <option value="">Select position</option>
                      {series?.team_format === 'two_team' ? (
                        <>
                          <option value="first_speaker">First Speaker</option>
                          <option value="second_speaker">Second Speaker</option>
                          <option value="reply_speaker">Reply Speaker</option>
                        </>
                      ) : (
                        <>
                          <option value="prime_minister">Prime Minister</option>
                          <option value="deputy_prime_minister">Deputy Prime Minister</option>
                          <option value="leader_of_opposition">Leader of Opposition</option>
                          <option value="deputy_leader_of_opposition">Deputy Leader of Opposition</option>
                          <option value="member_for_government">Member for Government</option>
                          <option value="government_whip">Government Whip</option>
                          <option value="member_for_opposition">Member for Opposition</option>
                          <option value="opposition_whip">Opposition Whip</option>
                        </>
                      )}
                    </select>
                  </div>
                </>
              )}
            </div>

            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() => setShowGuestModal(false)}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200"
              >
                Cancel
              </button>
              <button
                onClick={handleAddGuest}
                disabled={!guestName.trim()}
                className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50"
              >
                Add Guest
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Four Team Layout Component (British Parliamentary)
interface FourTeamLayoutProps {
  teams: MatchTeam[];
  draggedUser: DragItem | null;
  onDrop: (teamId: string, twoTeamRole?: TwoTeamSpeakerRole, fourTeamRole?: FourTeamSpeakerRole) => void;
  onRemove: (id: string) => void;
}

const fourTeamPositionOrder: FourTeamPosition[] = [
  'opening_government', 
  'opening_opposition', 
  'closing_government', 
  'closing_opposition'
];

const fourTeamPositionNames: Record<FourTeamPosition, string> = {
  'opening_government': 'Opening Government',
  'opening_opposition': 'Opening Opposition',
  'closing_government': 'Closing Government',
  'closing_opposition': 'Closing Opposition',
};

const fourTeamPositionColors: Record<FourTeamPosition, string> = {
  'opening_government': 'border-blue-300 bg-blue-50',
  'opening_opposition': 'border-red-300 bg-red-50',
  'closing_government': 'border-blue-200 bg-blue-100/50',
  'closing_opposition': 'border-red-200 bg-red-100/50',
};

const fourTeamSpeakerRoles: Record<FourTeamPosition, FourTeamSpeakerRole[]> = {
  'opening_government': ['prime_minister', 'deputy_prime_minister'],
  'opening_opposition': ['leader_of_opposition', 'deputy_leader_of_opposition'],
  'closing_government': ['member_of_government', 'government_whip'],
  'closing_opposition': ['member_of_opposition', 'opposition_whip'],
};

function FourTeamLayout({ teams, draggedUser, onDrop, onRemove }: FourTeamLayoutProps) {
  return (
    <div className="grid grid-cols-2 gap-4">
      {fourTeamPositionOrder.map(position => {
        const team = teams.find(t => t.four_team_position === position);
        const roles = fourTeamSpeakerRoles[position];
        
        return (
          <div key={position} className={`rounded-lg border-2 p-4 ${fourTeamPositionColors[position]}`}>
            <h3 className="font-medium text-sm mb-3">{fourTeamPositionNames[position]}</h3>
            
            <div className="space-y-2">
              {roles.map((role) => {
                const speaker = team?.speakers.find(s => s.four_team_speaker_role === role);
                
                return (
                  <div
                    key={role}
                    onDragOver={(e) => {
                      if (draggedUser && !speaker && team) {
                        e.preventDefault();
                      }
                    }}
                    onDrop={() => team && !speaker && onDrop(team.id, undefined, role)}
                    className={`min-h-[60px] rounded border-2 border-dashed flex items-center justify-center transition-colors ${
                      draggedUser && !speaker && team
                        ? 'border-blue-400 bg-blue-100'
                        : speaker
                        ? 'border-solid border-gray-300 bg-white'
                        : 'border-gray-300 bg-white/50'
                    }`}
                  >
                    {speaker ? (
                      <div className="flex items-center justify-between w-full px-3 py-2">
                        <div>
                          <div className="text-sm font-medium">{speaker.username}</div>
                          <div className="text-xs text-gray-500 capitalize">
                            {role.replace(/_/g, ' ')}
                          </div>
                        </div>
                        <button
                          onClick={() => onRemove(speaker.allocation_id)}
                          className="text-red-500 hover:text-red-700 p-1"
                        >
                          ×
                        </button>
                      </div>
                    ) : (
                      <span className="text-sm text-gray-400 capitalize">
                        {role.replace(/_/g, ' ')}
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// Two Team Layout Component
interface TwoTeamLayoutProps {
  teams: MatchTeam[];
  draggedUser: DragItem | null;
  onDrop: (teamId: string, twoTeamRole?: TwoTeamSpeakerRole, fourTeamRole?: FourTeamSpeakerRole) => void;
  onRemove: (id: string) => void;
}

const twoTeamPositionNames: Record<TwoTeamPosition, string> = {
  'government': 'Government',
  'opposition': 'Opposition',
};

const twoTeamPositionColors: Record<TwoTeamPosition, string> = {
  'government': 'border-blue-300 bg-blue-50',
  'opposition': 'border-red-300 bg-red-50',
};

const twoTeamSpeakerRoles: Record<TwoTeamPosition, TwoTeamSpeakerRole[]> = {
  'government': ['prime_minister', 'deputy_prime_minister', 'government_whip'],
  'opposition': ['leader_of_opposition', 'deputy_leader_of_opposition', 'opposition_whip'],
};

function TwoTeamLayout({ teams, draggedUser, onDrop, onRemove }: TwoTeamLayoutProps) {
  const positions: TwoTeamPosition[] = ['government', 'opposition'];

  return (
    <div className="grid grid-cols-2 gap-4">
      {positions.map(position => {
        const team = teams.find(t => t.two_team_position === position);
        const roles = twoTeamSpeakerRoles[position];
        
        return (
          <div key={position} className={`rounded-lg border-2 p-4 ${twoTeamPositionColors[position]}`}>
            <h3 className="font-medium text-sm mb-3">{twoTeamPositionNames[position]}</h3>
            
            <div className="space-y-2">
              {roles.map((role) => {
                const speaker = team?.speakers.find(s => s.two_team_speaker_role === role);
                
                return (
                  <div
                    key={role}
                    onDragOver={(e) => {
                      if (draggedUser && !speaker && team) {
                        e.preventDefault();
                      }
                    }}
                    onDrop={() => team && !speaker && onDrop(team.id, role, undefined)}
                    className={`min-h-[60px] rounded border-2 border-dashed flex items-center justify-center transition-colors ${
                      draggedUser && !speaker && team
                        ? 'border-blue-400 bg-blue-100'
                        : speaker
                        ? 'border-solid border-gray-300 bg-white'
                        : 'border-gray-300 bg-white/50'
                    }`}
                  >
                    {speaker ? (
                      <div className="flex items-center justify-between w-full px-3 py-2">
                        <div>
                          <div className="text-sm font-medium">{speaker.username}</div>
                          <div className="text-xs text-gray-500 capitalize">
                            {role.replace(/_/g, ' ')}
                          </div>
                        </div>
                        <button
                          onClick={() => onRemove(speaker.allocation_id)}
                          className="text-red-500 hover:text-red-700 p-1"
                        >
                          ×
                        </button>
                      </div>
                    ) : (
                      <span className="text-sm text-gray-400 capitalize">
                        {role.replace(/_/g, ' ')}
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// Adjudicator Drop Zone Component
interface AdjudicatorDropZoneProps {
  title: string;
  adjudicators: AdjudicatorInfo[];
  draggedUser: DragItem | null;
  onDrop: () => void;
  onRemove: (id: string) => void;
  className?: string;
}

function AdjudicatorDropZone({ title, adjudicators, draggedUser, onDrop, onRemove, className = '' }: AdjudicatorDropZoneProps) {
  return (
    <div className={`rounded-lg border-2 p-4 ${className}`}>
      <h3 className="font-medium text-sm mb-3 flex items-center justify-between">
        {title}
        <span className="text-xs text-gray-500">{adjudicators.length} assigned</span>
      </h3>
      
      <div
        onDragOver={(e) => {
          if (draggedUser) {
            e.preventDefault();
          }
        }}
        onDrop={onDrop}
        className={`min-h-[100px] rounded border-2 border-dashed p-2 transition-colors ${
          draggedUser ? 'border-blue-400 bg-blue-50' : 'border-gray-300'
        }`}
      >
        {adjudicators.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm text-gray-400">
            Drop adjudicators here
          </div>
        ) : (
          <div className="space-y-2">
            {adjudicators.map(adj => (
              <div
                key={adj.allocation_id}
                className="flex items-center justify-between bg-white rounded px-3 py-2 shadow-sm"
              >
                <div>
                  <div className="text-sm font-medium flex items-center gap-2">
                    {adj.username}
                    {adj.is_chair && (
                      <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-0.5 rounded">
                        Chair
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-gray-500">
                    {adj.has_submitted ? '✓ Ballot submitted' : 'Ballot pending'}
                  </div>
                </div>
                <button
                  onClick={() => onRemove(adj.allocation_id)}
                  className="text-red-500 hover:text-red-700 p-1"
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

// Resource Drop Zone Component
interface ResourceDropZoneProps {
  teams: MatchTeam[];
  draggedUser: DragItem | null;
  onDrop: () => void;
  onRemove: (id: string) => void;
}

function ResourceDropZone({ teams, draggedUser, onDrop, onRemove }: ResourceDropZoneProps) {
  // Collect all resources from all teams
  const allResources = teams.flatMap(t => t.resources);

  return (
    <div
      onDragOver={(e) => {
        if (draggedUser) {
          e.preventDefault();
        }
      }}
      onDrop={onDrop}
      className={`min-h-[100px] rounded-lg border-2 border-dashed p-4 transition-colors ${
        draggedUser ? 'border-blue-400 bg-blue-50' : 'border-gray-300 bg-gray-50'
      }`}
    >
      {allResources.length === 0 ? (
        <div className="flex items-center justify-center h-full text-sm text-gray-400">
          Drop resource personnel here
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
          {allResources.map(resource => (
            <div
              key={resource.allocation_id}
              className="flex items-center justify-between bg-white rounded px-3 py-2 shadow-sm"
            >
              <div className="text-sm font-medium">{resource.username}</div>
              <button
                onClick={() => onRemove(resource.allocation_id)}
                className="text-red-500 hover:text-red-700 p-1"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// Results Tab Component
interface ResultsTabProps {
  match: MatchResponse;
  ballots: BallotResponse[];
  onToggleRelease: (field: 'scores_released' | 'rankings_released') => void;
}

function ResultsTab({ match, ballots, onToggleRelease }: ResultsTabProps) {
  // Calculate average scores per speaker
  const speakerScoreMap = new Map<string, { total: number; count: number; feedback: string[] }>();
  
  ballots.forEach(ballot => {
    ballot.speaker_scores.forEach(score => {
      const existing = speakerScoreMap.get(score.allocation_id) || { total: 0, count: 0, feedback: [] };
      existing.total += score.score;
      existing.count += 1;
      if (score.feedback) {
        existing.feedback.push(score.feedback);
      }
      speakerScoreMap.set(score.allocation_id, existing);
    });
  });

  // Calculate team rankings
  const teamRankMap = new Map<string, number[]>();
  ballots.forEach(ballot => {
    ballot.team_rankings.forEach(ranking => {
      const existing = teamRankMap.get(ranking.team_id) || [];
      existing.push(ranking.rank);
      teamRankMap.set(ranking.team_id, existing);
    });
  });

  return (
    <div className="space-y-6">
      {/* Release Controls */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold mb-4">Score Release Settings</h2>
        
        <div className="flex items-center gap-8">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={match.scores_released}
              onChange={() => onToggleRelease('scores_released')}
              className="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="font-medium">Release Scores to Participants</span>
          </label>
          
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={match.rankings_released}
              onChange={() => onToggleRelease('rankings_released')}
              className="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="font-medium">Release Rankings to Participants</span>
          </label>
        </div>
      </div>

      {/* Motion Info */}
      {match.motion && (
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold mb-2">Motion</h2>
          <p className="text-gray-700">{match.motion}</p>
          {match.info_slide && (
            <div className="mt-4 p-4 bg-gray-50 rounded-lg">
              <h3 className="text-sm font-medium text-gray-500 mb-1">Info Slide</h3>
              <p className="text-gray-700">{match.info_slide}</p>
            </div>
          )}
        </div>
      )}

      {/* Team Results */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold mb-4">Team Results</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {match.teams.map(team => {
            const ranks = teamRankMap.get(team.id) || [];
            const avgRank = ranks.length > 0 
              ? (ranks.reduce((a, b) => a + b, 0) / ranks.length).toFixed(1)
              : null;

            return (
              <div 
                key={team.id} 
                className={`rounded-lg p-4 border-2 ${
                  team.final_rank === 1 ? 'border-yellow-400 bg-yellow-50' :
                  team.final_rank === 2 ? 'border-gray-400 bg-gray-50' :
                  team.final_rank === 3 ? 'border-orange-400 bg-orange-50' :
                  'border-gray-200 bg-white'
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium text-sm">
                    {team.four_team_position?.replace(/_/g, ' ').toUpperCase() || 
                     team.two_team_position?.replace(/_/g, ' ').toUpperCase() ||
                     team.team_name || 'Team'}
                  </h3>
                  {team.final_rank && (
                    <span className={`text-xl font-bold ${
                      team.final_rank === 1 ? 'text-yellow-600' :
                      team.final_rank === 2 ? 'text-gray-600' :
                      team.final_rank === 3 ? 'text-orange-600' :
                      'text-gray-400'
                    }`}>
                      #{team.final_rank}
                    </span>
                  )}
                </div>
                
                {avgRank && (
                  <div className="text-sm text-gray-500 mb-2">
                    Avg Rank: {avgRank}
                  </div>
                )}
                
                <div className="text-sm text-gray-500 mb-3">
                  Total Points: {team.total_speaker_points ? Number(team.total_speaker_points).toFixed(1) : '—'}
                </div>

                {/* Speakers in this team */}
                <div className="space-y-2">
                  {team.speakers.map(speaker => {
                    const scoreData = speakerScoreMap.get(speaker.allocation_id);
                    const avgScore = scoreData && scoreData.count > 0
                      ? (scoreData.total / scoreData.count).toFixed(1)
                      : null;

                    return (
                      <div key={speaker.allocation_id} className="flex justify-between items-center text-sm">
                        <span>{speaker.username}</span>
                        <span className="font-medium text-blue-600">
                          {avgScore || speaker.score?.toFixed(1) || '—'}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Ballot Details */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold mb-4">
          Ballot Submissions ({ballots.length})
        </h2>
        
        {ballots.length === 0 ? (
          <p className="text-gray-500 text-center py-4">
            No ballots have been submitted yet.
          </p>
        ) : (
          <div className="space-y-4">
            {ballots.map(ballot => (
              <div key={ballot.id} className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="font-medium">{ballot.adjudicator_username}</div>
                  <div className="text-sm text-gray-500">
                    {ballot.is_voting ? 'Voting' : 'Non-Voting'} • 
                    {ballot.is_submitted 
                      ? ` Submitted ${ballot.submitted_at ? new Date(ballot.submitted_at).toLocaleString() : ''}` 
                      : ' Draft'}
                  </div>
                </div>
                
                {ballot.notes && (
                  <div className="mb-3 p-3 bg-gray-50 rounded text-sm text-gray-700">
                    {ballot.notes}
                  </div>
                )}

                <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                  {ballot.speaker_scores.map(score => (
                    <div key={score.id} className="text-sm">
                      <span className="text-gray-500">{score.speaker_username}:</span>{' '}
                      <span className="font-medium">{score.score}</span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
