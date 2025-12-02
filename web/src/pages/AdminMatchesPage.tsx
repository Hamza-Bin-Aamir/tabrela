import { useState, useEffect, useCallback } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { TabulationService } from '../services/tabulation';
import { AttendanceService } from '../services/attendance';
import type { 
  MatchSeries, 
  MatchResponse, 
  Event, 
  TeamFormat,
  CreateSeriesRequest,
  CreateMatchRequest,
} from '../services/types';

export default function AdminMatchesPage() {
  const { eventId } = useParams<{ eventId: string }>();
  const navigate = useNavigate();

  const [event, setEvent] = useState<Event | null>(null);
  const [series, setSeries] = useState<MatchSeries | null>(null);
  const [matches, setMatches] = useState<MatchResponse[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Modals
  const [showCreateMatch, setShowCreateMatch] = useState(false);

  // Form states for initial setup
  const [setupFormat, setSetupFormat] = useState<TeamFormat>('two_team');
  const [setupReply, setSetupReply] = useState(false);

  const [newMatchRoom, setNewMatchRoom] = useState('');
  const [newMatchMotion, setNewMatchMotion] = useState('');

  const loadData = useCallback(async () => {
    if (!eventId) return;

    setIsLoading(true);
    setError(null);

    try {
      // Load event details
      const eventData = await AttendanceService.getEvent(eventId);
      setEvent(eventData);

      // Load series for this event - for friendly matches, there should be only one
      const seriesResponse = await TabulationService.listSeries(eventId);
      
      if (seriesResponse.series.length > 0) {
        // Use the first (and should be only) series
        const mainSeries = seriesResponse.series[0];
        setSeries(mainSeries);
        
        // Load matches for this series
        const matchesResponse = await TabulationService.listMatches({ 
          seriesId: mainSeries.id 
        });
        setMatches(matchesResponse.matches);
      } else {
        // No series exists yet - show setup modal
        setSeries(null);
        setMatches([]);
      }
    } catch (err) {
      setError('Failed to load data. Please try again.');
      console.error('Failed to load data:', err);
    } finally {
      setIsLoading(false);
    }
  }, [eventId]);

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

  const handleSetup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!eventId || !event) return;

    try {
      const data: CreateSeriesRequest = {
        event_id: eventId,
        name: `${event.title} Matches`,
        team_format: setupFormat,
        allow_reply_speeches: setupReply,
        is_break_round: false,
      };

      const response = await TabulationService.createSeries(data);
      setSeries(response.series);
      setSuccessMessage('Match setup complete!');
      await loadData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to setup matches');
    }
  };

  const handleCreateMatch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!series) return;

    try {
      const data: CreateMatchRequest = {
        series_id: series.id,
        room_name: newMatchRoom || undefined,
        motion: newMatchMotion || undefined,
      };

      await TabulationService.createMatch(data);
      setSuccessMessage('Match created successfully!');
      setShowCreateMatch(false);
      setNewMatchRoom('');
      setNewMatchMotion('');
      
      // Reload matches
      const matchesResponse = await TabulationService.listMatches({ 
        seriesId: series.id 
      });
      setMatches(matchesResponse.matches);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to create match');
    }
  };

  const handleDeleteMatch = async (matchId: string) => {
    if (!confirm('Are you sure you want to delete this match?')) {
      return;
    }

    try {
      await TabulationService.deleteMatch(matchId);
      setSuccessMessage('Match deleted successfully!');
      setMatches(matches.filter(m => m.id !== matchId));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to delete match');
    }
  };

  const handleUpdateMatchStatus = async (matchId: string, status: 'draft' | 'published' | 'in_progress' | 'completed' | 'cancelled') => {
    try {
      await TabulationService.updateMatch(matchId, { status });
      setSuccessMessage(`Match status updated to ${status.replace('_', ' ')}!`);
      setMatches(matches.map(m => m.id === matchId ? { ...m, status } : m));
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update match status');
    }
  };

  const getStatusBadge = (status: string) => {
    const badges: Record<string, { bg: string; text: string; label: string }> = {
      draft: { bg: 'bg-gray-100', text: 'text-gray-800', label: 'Draft' },
      published: { bg: 'bg-blue-100', text: 'text-blue-800', label: 'Published' },
      in_progress: { bg: 'bg-yellow-100', text: 'text-yellow-800', label: 'In Progress' },
      completed: { bg: 'bg-green-100', text: 'text-green-800', label: 'Completed' },
      cancelled: { bg: 'bg-red-100', text: 'text-red-800', label: 'Cancelled' },
    };
    const badge = badges[status] || badges.draft;
    return (
      <span className={`px-2 py-1 text-xs font-medium rounded-full ${badge.bg} ${badge.text}`}>
        {badge.label}
      </span>
    );
  };

  const getFormatLabel = (format: TeamFormat) => {
    return format === 'two_team' ? '2-Team (1v1)' : '4-Team (BP)';
  };

  const handleAllocate = () => {
    if (!eventId || !series) return;
    navigate(`/admin/events/${eventId}/series/${series.id}/allocate`);
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

  // Show setup screen if no series exists
  if (!series) {
    return (
      <div className="min-h-screen bg-gray-50 py-8">
        <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
          {/* Header */}
          <div className="mb-8">
            <nav className="flex mb-4" aria-label="Breadcrumb">
              <ol className="flex items-center space-x-2 text-sm text-gray-500">
                <li><Link to="/events" className="hover:text-gray-700">Events</Link></li>
                <li><span className="mx-2">/</span></li>
                <li><Link to={`/events/${eventId}`} className="hover:text-gray-700">{event?.title}</Link></li>
                <li><span className="mx-2">/</span></li>
                <li className="text-gray-900 font-medium">Matches</li>
              </ol>
            </nav>
            <h1 className="text-3xl font-bold text-gray-900">Match Setup</h1>
            <p className="mt-2 text-gray-600">Configure match settings for {event?.title}</p>
          </div>

          {error && (
            <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
              <p className="text-sm text-red-600">{error}</p>
            </div>
          )}

          {/* Setup Card */}
          <div className="bg-white rounded-lg shadow-lg p-8">
            <div className="text-center mb-8">
              <div className="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-indigo-100 mb-4">
                <svg className="h-8 w-8 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              </div>
              <h2 className="text-xl font-semibold text-gray-900">Setup Matches</h2>
              <p className="mt-2 text-gray-500">Choose the format for your friendly matches</p>
            </div>

            <form onSubmit={handleSetup} className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">Team Format</label>
                <div className="grid grid-cols-2 gap-4">
                  <label className={`relative flex cursor-pointer rounded-lg border p-4 focus:outline-none ${
                    setupFormat === 'two_team' ? 'border-indigo-600 ring-2 ring-indigo-600' : 'border-gray-300'
                  }`}>
                    <input
                      type="radio"
                      name="format"
                      value="two_team"
                      checked={setupFormat === 'two_team'}
                      onChange={() => setSetupFormat('two_team')}
                      className="sr-only"
                    />
                    <div className="flex flex-col">
                      <span className="block text-sm font-medium text-gray-900">2-Team (1v1)</span>
                      <span className="mt-1 text-sm text-gray-500">Government vs Opposition</span>
                    </div>
                  </label>
                  <label className={`relative flex cursor-pointer rounded-lg border p-4 focus:outline-none ${
                    setupFormat === 'four_team' ? 'border-indigo-600 ring-2 ring-indigo-600' : 'border-gray-300'
                  }`}>
                    <input
                      type="radio"
                      name="format"
                      value="four_team"
                      checked={setupFormat === 'four_team'}
                      onChange={() => setSetupFormat('four_team')}
                      className="sr-only"
                    />
                    <div className="flex flex-col">
                      <span className="block text-sm font-medium text-gray-900">4-Team (BP)</span>
                      <span className="mt-1 text-sm text-gray-500">British Parliamentary</span>
                    </div>
                  </label>
                </div>
              </div>

              <div>
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={setupReply}
                    onChange={(e) => setSetupReply(e.target.checked)}
                    className="rounded border-gray-300 text-indigo-600 focus:ring-indigo-500"
                  />
                  <span className="ml-2 text-sm text-gray-700">Allow Reply Speeches</span>
                </label>
              </div>

              <button
                type="submit"
                className="w-full flex justify-center py-3 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                Continue to Matches
              </button>
            </form>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <nav className="flex mb-4" aria-label="Breadcrumb">
            <ol className="flex items-center space-x-2 text-sm text-gray-500">
              <li><Link to="/events" className="hover:text-gray-700">Events</Link></li>
              <li><span className="mx-2">/</span></li>
              <li><Link to={`/events/${eventId}`} className="hover:text-gray-700">{event?.title}</Link></li>
              <li><span className="mx-2">/</span></li>
              <li className="text-gray-900 font-medium">Matches</li>
            </ol>
          </nav>

          <div className="flex justify-between items-start">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Matches</h1>
              <p className="mt-2 text-sm text-gray-600">
                {getFormatLabel(series.team_format)} • {matches.length} matches
                {series.allow_reply_speeches && ' • Reply speeches enabled'}
              </p>
            </div>
            <div className="flex space-x-3">
              <button
                onClick={handleAllocate}
                className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
              >
                <svg className="h-5 w-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                </svg>
                Allocate People
              </button>
              <button
                onClick={() => setShowCreateMatch(true)}
                className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
              >
                <svg className="h-5 w-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
                New Match
              </button>
            </div>
          </div>
        </div>

        {/* Messages */}
        {error && (
          <div className="mb-4 bg-red-50 border border-red-200 rounded-md p-4">
            <div className="flex justify-between">
              <p className="text-sm text-red-600">{error}</p>
              <button onClick={() => setError(null)} className="text-red-400 hover:text-red-600">
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
        )}
        {successMessage && (
          <div className="mb-4 bg-green-50 border border-green-200 rounded-md p-4">
            <p className="text-sm text-green-600">{successMessage}</p>
          </div>
        )}

        {/* Matches Grid */}
        {matches.length === 0 ? (
          <div className="bg-white rounded-lg shadow-sm p-12 text-center">
            <div className="mx-auto flex items-center justify-center h-16 w-16 rounded-full bg-gray-100 mb-4">
              <svg className="h-8 w-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No matches yet</h3>
            <p className="text-gray-500 mb-6">Create your first match to get started</p>
            <button
              onClick={() => setShowCreateMatch(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
            >
              Create Match
            </button>
          </div>
        ) : (
          <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
            {matches.map((match, index) => (
              <div key={match.id} className="bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow">
                <div className="p-5">
                  {/* Header */}
                  <div className="flex justify-between items-start mb-4">
                    <div>
                      <h3 className="font-semibold text-lg text-gray-900">
                        {match.room_name || `Match ${index + 1}`}
                      </h3>
                      {getStatusBadge(match.status)}
                    </div>
                    <button
                      onClick={() => handleDeleteMatch(match.id)}
                      className="text-gray-400 hover:text-red-600 p-1"
                      title="Delete match"
                    >
                      <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>

                  {/* Motion */}
                  {match.motion && (
                    <p className="text-sm text-gray-600 mb-4 line-clamp-2 italic">"{match.motion}"</p>
                  )}

                  {/* Teams */}
                  <div className="space-y-3 mb-4">
                    {match.teams.map((team) => (
                      <div key={team.id} className="bg-gray-50 rounded-lg p-3">
                        <div className="flex items-center justify-between mb-1">
                          <span className="text-sm font-medium text-gray-700">
                            {team.two_team_position === 'government' ? '🏛️ Government' :
                             team.two_team_position === 'opposition' ? '⚔️ Opposition' :
                             team.four_team_position?.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase()) || 'Team'}
                          </span>
                          {team.final_rank && match.rankings_released && (
                            <span className="text-xs bg-indigo-100 text-indigo-700 px-2 py-0.5 rounded">
                              Rank {team.final_rank}
                            </span>
                          )}
                        </div>
                        <div className="text-sm text-gray-600">
                          {team.speakers.length > 0 ? (
                            <ul className="space-y-1">
                              {team.speakers.map((speaker) => (
                                <li key={speaker.allocation_id} className="flex items-center">
                                  <span className={speaker.was_checked_in ? 'text-gray-900' : 'text-gray-400 italic'}>
                                    {speaker.username}
                                  </span>
                                  {!speaker.was_checked_in && (
                                    <span className="ml-1 text-xs text-amber-600">(not checked in)</span>
                                  )}
                                  {speaker.score && match.scores_released && (
                                    <span className="ml-auto text-gray-500">{Number(speaker.score).toFixed(1)} pts</span>
                                  )}
                                </li>
                              ))}
                            </ul>
                          ) : (
                            <span className="text-gray-400 italic">No speakers assigned</span>
                          )}
                        </div>
                      </div>
                    ))}

                    {/* Show empty slots if no teams */}
                    {match.teams.length === 0 && (
                      <div className="text-center py-4 text-gray-400 text-sm">
                        No teams allocated yet
                      </div>
                    )}
                  </div>

                  {/* Adjudicators */}
                  <div className="border-t border-gray-100 pt-3 mb-4">
                    <span className="text-xs font-medium text-gray-500 uppercase">Adjudicators</span>
                    {match.adjudicators.length > 0 ? (
                      <ul className="mt-1 text-sm">
                        {match.adjudicators.map((adj) => (
                          <li key={adj.allocation_id} className="flex items-center justify-between">
                            <span className={adj.was_checked_in ? '' : 'text-gray-400 italic'}>
                              {adj.username}
                              {adj.is_chair && <span className="ml-1 text-indigo-600">(Chair)</span>}
                              {!adj.is_voting && <span className="ml-1 text-gray-400">(Shadow)</span>}
                              {!adj.was_checked_in && <span className="ml-1 text-xs text-amber-600">(not checked in)</span>}
                            </span>
                            {adj.has_submitted && (
                              <span className="text-green-600">✓</span>
                            )}
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p className="mt-1 text-sm text-gray-400 italic">No adjudicators assigned</p>
                    )}
                  </div>

                  {/* Stats & Actions */}
                  <div className="flex items-center justify-between pt-3 border-t border-gray-100">
                    <div className="text-xs text-gray-500">
                      {match.adjudicators.filter(a => a.has_submitted).length}/{match.adjudicators.filter(a => a.is_voting).length} ballots
                    </div>
                    <div className="flex items-center gap-2">
                      {/* Status toggle button */}
                      {match.status === 'draft' && (
                        <button
                          onClick={() => handleUpdateMatchStatus(match.id, 'published')}
                          className="text-xs px-2 py-1 bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
                          title="Publish match"
                        >
                          Publish
                        </button>
                      )}
                      {match.status === 'published' && (
                        <button
                          onClick={() => handleUpdateMatchStatus(match.id, 'in_progress')}
                          className="text-xs px-2 py-1 bg-yellow-100 text-yellow-700 rounded hover:bg-yellow-200"
                          title="Start match"
                        >
                          Start
                        </button>
                      )}
                      {match.status === 'in_progress' && (
                        <button
                          onClick={() => handleUpdateMatchStatus(match.id, 'completed')}
                          className="text-xs px-2 py-1 bg-green-100 text-green-700 rounded hover:bg-green-200"
                          title="Mark as completed"
                        >
                          Complete
                        </button>
                      )}
                      <Link
                        to={`/admin/matches/${match.id}`}
                        className="text-sm font-medium text-indigo-600 hover:text-indigo-800"
                      >
                        Manage →
                      </Link>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Create Match Modal */}
        {showCreateMatch && series && (
          <div className="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
            <div className="relative top-20 mx-auto p-5 border w-full max-w-md shadow-lg rounded-md bg-white">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-lg font-medium text-gray-900">Create New Match</h3>
                <button onClick={() => setShowCreateMatch(false)} className="text-gray-400 hover:text-gray-600">
                  <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>

              <form onSubmit={handleCreateMatch} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700">Room Name (Optional)</label>
                  <input
                    type="text"
                    value={newMatchRoom}
                    onChange={(e) => setNewMatchRoom(e.target.value)}
                    placeholder="e.g., Room 1, Main Hall"
                    className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700">Motion (Optional)</label>
                  <textarea
                    value={newMatchMotion}
                    onChange={(e) => setNewMatchMotion(e.target.value)}
                    placeholder="This House believes that..."
                    rows={3}
                    className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                  />
                </div>

                <p className="text-sm text-gray-500">
                  Teams will be created automatically ({getFormatLabel(series.team_format)}).
                  You can allocate speakers afterwards.
                </p>

                <div className="flex justify-end space-x-3 pt-4">
                  <button
                    type="button"
                    onClick={() => setShowCreateMatch(false)}
                    className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700"
                  >
                    Create Match
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
