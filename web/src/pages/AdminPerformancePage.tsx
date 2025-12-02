import { useState, useEffect, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import { AttendanceService } from '../services/attendance';
import type { PerformanceResponse, Event } from '../services/types';

export default function AdminPerformancePage() {
  const { eventId } = useParams<{ eventId: string }>();
  
  const [event, setEvent] = useState<Event | null>(null);
  const [performances, setPerformances] = useState<PerformanceResponse[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'average_score' | 'win_rate' | 'total_rounds'>('total_rounds');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      
      // Get event details if eventId provided
      if (eventId) {
        const eventData = await AttendanceService.getEvent(eventId);
        setEvent(eventData);
      }
      
      // For now, we'd need an endpoint to list all user performances
      // This is a placeholder - in reality you'd fetch from a paginated endpoint
      // For demo purposes, we'll just show an empty state
      setPerformances([]);

      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load performance data');
    } finally {
      setLoading(false);
    }
  }, [eventId]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const filteredPerformances = performances.filter(p =>
    p.username.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const sortedPerformances = [...filteredPerformances].sort((a, b) => {
    let aVal: number = 0;
    let bVal: number = 0;
    
    switch (sortBy) {
      case 'average_score':
        aVal = a.average_speaker_score || 0;
        bVal = b.average_speaker_score || 0;
        break;
      case 'win_rate':
        aVal = a.win_rate || 0;
        bVal = b.win_rate || 0;
        break;
      case 'total_rounds':
        aVal = a.total_rounds;
        bVal = b.total_rounds;
        break;
    }
    
    return sortOrder === 'asc' ? aVal - bVal : bVal - aVal;
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between">
            <div>
              {event && (
                <Link 
                  to={`/admin/events/${eventId}/matches`}
                  className="text-sm text-gray-500 hover:text-gray-700"
                >
                  ← Back to Matches
                </Link>
              )}
              <h1 className="text-2xl font-bold text-gray-900 mt-1">
                Performance Statistics
              </h1>
              {event && (
                <p className="text-sm text-gray-500">{event.title}</p>
              )}
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
        {error && (
          <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
            {error}
          </div>
        )}

        {/* Filters */}
        <div className="bg-white rounded-lg shadow p-4 mb-6">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1">
              <input
                type="text"
                placeholder="Search participants..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
            
            <div className="flex gap-4">
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
                className="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                <option value="total_rounds">Sort by Total Rounds</option>
                <option value="average_score">Sort by Avg Score</option>
                <option value="win_rate">Sort by Win Rate</option>
              </select>
              
              <button
                onClick={() => setSortOrder(prev => prev === 'asc' ? 'desc' : 'asc')}
                className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                {sortOrder === 'desc' ? '↓ Desc' : '↑ Asc'}
              </button>
            </div>
          </div>
        </div>

        {/* Stats Overview */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <div className="bg-white rounded-lg shadow p-4">
            <div className="text-sm text-gray-500">Total Participants</div>
            <div className="text-2xl font-bold text-gray-900">{performances.length}</div>
          </div>
          <div className="bg-white rounded-lg shadow p-4">
            <div className="text-sm text-gray-500">Active Speakers</div>
            <div className="text-2xl font-bold text-blue-600">
              {performances.filter(p => p.rounds_as_speaker > 0).length}
            </div>
          </div>
          <div className="bg-white rounded-lg shadow p-4">
            <div className="text-sm text-gray-500">Active Adjudicators</div>
            <div className="text-2xl font-bold text-green-600">
              {performances.filter(p => p.rounds_as_adjudicator > 0).length}
            </div>
          </div>
          <div className="bg-white rounded-lg shadow p-4">
            <div className="text-sm text-gray-500">Avg Speaker Score</div>
            <div className="text-2xl font-bold text-purple-600">
              {performances.length > 0
                ? (performances.reduce((sum, p) => sum + (p.average_speaker_score || 0), 0) / performances.filter(p => p.average_speaker_score).length).toFixed(1)
                : '—'}
            </div>
          </div>
        </div>

        {/* Performance Table */}
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Participant
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Total Rounds
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Speaker / Adjudicator
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Avg Score
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Win Rate
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Rankings
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {sortedPerformances.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="px-6 py-12 text-center text-gray-500">
                      No performance data available yet.
                      {searchQuery && ' Try adjusting your search.'}
                    </td>
                  </tr>
                ) : (
                  sortedPerformances.map((perf) => (
                    <tr key={perf.user_id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <Link 
                          to={`/admin/users/${perf.user_id}/performance${eventId ? `?event_id=${eventId}` : ''}`}
                          className="text-blue-600 hover:text-blue-800 font-medium"
                        >
                          {perf.username}
                        </Link>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {perf.total_rounds}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {perf.rounds_as_speaker} / {perf.rounds_as_adjudicator}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {perf.average_speaker_score !== null ? (
                          <span className={`font-medium ${
                            perf.average_speaker_score >= 78 ? 'text-green-600' :
                            perf.average_speaker_score >= 74 ? 'text-blue-600' :
                            'text-gray-600'
                          }`}>
                            {perf.average_speaker_score.toFixed(1)}
                          </span>
                        ) : (
                          <span className="text-gray-400">—</span>
                        )}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        {perf.win_rate !== null ? (
                          <span className={`font-medium ${
                            perf.win_rate >= 0.5 ? 'text-green-600' : 'text-gray-600'
                          }`}>
                            {(perf.win_rate * 100).toFixed(0)}%
                          </span>
                        ) : (
                          <span className="text-gray-400">—</span>
                        )}
                        <span className="text-xs text-gray-500 ml-1">
                          ({perf.total_wins}W / {perf.total_losses}L)
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <div className="flex gap-1">
                          {perf.rankings.map((r) => (
                            <span 
                              key={r.rank}
                              className={`text-xs px-2 py-0.5 rounded ${
                                r.rank === 1 ? 'bg-yellow-100 text-yellow-800' :
                                r.rank === 2 ? 'bg-gray-100 text-gray-800' :
                                r.rank === 3 ? 'bg-orange-100 text-orange-800' :
                                'bg-gray-50 text-gray-600'
                              }`}
                            >
                              {r.rank}st: {r.count}
                            </span>
                          ))}
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </main>
    </div>
  );
}
