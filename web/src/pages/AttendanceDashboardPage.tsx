import { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { AttendanceService } from '../services/attendance';
import { AdminService } from '../services/admin';
import type {
  AttendanceMatrixResponse,
  EventSummary,
  AttendanceMatrixRow,
  AttendanceCellStatus,
  AggregateStats,
} from '../services/types';

export default function AttendanceDashboardPage() {
  const navigate = useNavigate();
  const [data, setData] = useState<AttendanceMatrixResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isAdmin, setIsAdmin] = useState<boolean | null>(null);

  useEffect(() => {
    const checkAdminAndLoad = async () => {
      try {
        const response = await AdminService.checkAdminStatus();
        setIsAdmin(response.is_admin);
        if (!response.is_admin) {
          navigate('/events');
          return;
        }
        await loadData();
      } catch {
        setIsAdmin(false);
        navigate('/events');
      }
    };
    checkAdminAndLoad();
  }, [navigate]);

  const loadData = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await AttendanceService.getAttendanceMatrix();
      setData(response);
    } catch (err) {
      setError('Failed to load attendance data. Please try again.');
      console.error('Failed to load attendance matrix:', err);
    } finally {
      setIsLoading(false);
    }
  };

  if (isAdmin === null || isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <svg className="animate-spin h-12 w-12 text-indigo-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-red-600 mb-2">Error</h1>
          <p className="text-gray-600 mb-4">{error}</p>
          <button
            onClick={loadData}
            className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!data) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-full mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8">
          <Link to="/events" className="text-indigo-600 hover:text-indigo-800 mb-4 inline-block">
            ← Back to events
          </Link>
          <h1 className="text-3xl font-bold text-gray-900">Attendance Dashboard</h1>
          <p className="mt-2 text-gray-600">
            Comprehensive view of attendance across all events and users
          </p>
        </div>

        {/* Aggregate Stats Cards */}
        <AggregateStatsSection stats={data.aggregate_stats} />

        {/* Event Type Breakdown */}
        <EventTypeBreakdown eventTypes={data.aggregate_stats.events_by_type} />

        {/* Most Reliable Users */}
        <MostReliableUsers users={data.aggregate_stats.most_reliable_users} />

        {/* Attendance Matrix */}
        <AttendanceMatrix events={data.events} rows={data.rows} />
      </div>
    </div>
  );
}

// ============================================================================
// Sub-components
// ============================================================================

function AggregateStatsSection({ stats }: { stats: AggregateStats }) {
  return (
    <div className="mb-8">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Overview</h2>
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        <StatCard
          title="Total Events"
          value={stats.total_events.toString()}
          color="blue"
        />
        <StatCard
          title="Total Users"
          value={stats.total_users.toString()}
          color="indigo"
        />
        <StatCard
          title="Availability Rate"
          value={`${stats.overall_availability_rate.toFixed(1)}%`}
          color="green"
        />
        <StatCard
          title="Attendance Rate"
          value={`${stats.overall_attendance_rate.toFixed(1)}%`}
          color="emerald"
        />
        <StatCard
          title="Avg Available/Event"
          value={stats.avg_available_per_event.toFixed(1)}
          color="yellow"
        />
        <StatCard
          title="Avg Checked-in/Event"
          value={stats.avg_checked_in_per_event.toFixed(1)}
          color="orange"
        />
      </div>

      {/* Most/Least Attended Events */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
        {stats.most_attended_event && (
          <div className="bg-green-50 border border-green-200 rounded-lg p-4">
            <h3 className="text-sm font-medium text-green-800 mb-2">🏆 Most Attended Event</h3>
            <Link to={`/events/${stats.most_attended_event.id}`} className="text-green-900 font-semibold hover:underline">
              {stats.most_attended_event.title}
            </Link>
            <p className="text-sm text-green-700 mt-1">
              {stats.most_attended_event.total_checked_in} checked in / {stats.most_attended_event.total_available} available
            </p>
          </div>
        )}
        {stats.least_attended_event && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4">
            <h3 className="text-sm font-medium text-red-800 mb-2">📉 Least Attended Event</h3>
            <Link to={`/events/${stats.least_attended_event.id}`} className="text-red-900 font-semibold hover:underline">
              {stats.least_attended_event.title}
            </Link>
            <p className="text-sm text-red-700 mt-1">
              {stats.least_attended_event.total_checked_in} checked in / {stats.least_attended_event.total_available} available
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ title, value, color }: { title: string; value: string; color: string }) {
  const colorClasses: Record<string, string> = {
    blue: 'bg-blue-50 border-blue-200 text-blue-900',
    indigo: 'bg-indigo-50 border-indigo-200 text-indigo-900',
    green: 'bg-green-50 border-green-200 text-green-900',
    emerald: 'bg-emerald-50 border-emerald-200 text-emerald-900',
    yellow: 'bg-yellow-50 border-yellow-200 text-yellow-900',
    orange: 'bg-orange-50 border-orange-200 text-orange-900',
  };

  return (
    <div className={`rounded-lg border p-4 ${colorClasses[color] || colorClasses.blue}`}>
      <p className="text-sm font-medium opacity-75">{title}</p>
      <p className="text-2xl font-bold mt-1">{value}</p>
    </div>
  );
}

function EventTypeBreakdown({ eventTypes }: { eventTypes: { event_type: string; count: number; avg_attendance: number }[] }) {
  if (eventTypes.length === 0) return null;

  const formatEventType = (type: string) => {
    return type.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  };

  return (
    <div className="mb-8">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Attendance by Event Type</h2>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Event Type
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                # Events
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Avg Attendance
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {eventTypes.map((et) => (
              <tr key={et.event_type}>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                  {formatEventType(et.event_type)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {et.count}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  <div className="flex items-center">
                    <div className="w-16 bg-gray-200 rounded-full h-2 mr-2">
                      <div
                        className="bg-indigo-600 h-2 rounded-full"
                        style={{ width: `${Math.min(et.avg_attendance, 100)}%` }}
                      ></div>
                    </div>
                    {et.avg_attendance.toFixed(1)}%
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function MostReliableUsers({ users }: { users: { user_id: string; username: string; attendance_rate: number; events_checked_in: number; total_events: number }[] }) {
  if (users.length === 0) return null;

  return (
    <div className="mb-8">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">🌟 Top 5 Most Reliable Members</h2>
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        {users.map((user, index) => (
          <div key={user.user_id} className="bg-white rounded-lg shadow p-4 text-center">
            <div className="text-3xl mb-2">
              {index === 0 ? '🥇' : index === 1 ? '🥈' : index === 2 ? '🥉' : '⭐'}
            </div>
            <p className="font-semibold text-gray-900">{user.username}</p>
            <p className="text-2xl font-bold text-indigo-600 mt-1">
              {user.attendance_rate.toFixed(1)}%
            </p>
            <p className="text-xs text-gray-500 mt-1">
              {user.events_checked_in}/{user.total_events} events
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}

function AttendanceMatrix({ events, rows }: { events: EventSummary[]; rows: AttendanceMatrixRow[] }) {
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  const getCellStyle = (status: AttendanceCellStatus) => {
    switch (status) {
      case 'checked_in':
        return 'bg-green-500 text-white';
      case 'available':
        return 'bg-yellow-400 text-yellow-900';
      case 'unavailable':
        return 'bg-red-400 text-white';
      case 'no_response':
      default:
        return 'bg-gray-200 text-gray-500';
    }
  };

  const getCellLabel = (status: AttendanceCellStatus) => {
    switch (status) {
      case 'checked_in':
        return '✓';
      case 'available':
        return 'A';
      case 'unavailable':
        return '✗';
      case 'no_response':
      default:
        return '—';
    }
  };

  if (events.length === 0) {
    return (
      <div className="bg-white rounded-lg shadow p-8 text-center text-gray-500">
        No events found. Create some events to see the attendance matrix.
      </div>
    );
  }

  return (
    <div className="mb-8">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">Attendance Matrix</h2>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        {/* Legend */}
        <div className="px-4 py-3 bg-gray-50 border-b flex flex-wrap gap-4 text-sm">
          <span className="flex items-center gap-1">
            <span className="w-4 h-4 bg-green-500 rounded"></span> Checked In
          </span>
          <span className="flex items-center gap-1">
            <span className="w-4 h-4 bg-yellow-400 rounded"></span> Available
          </span>
          <span className="flex items-center gap-1">
            <span className="w-4 h-4 bg-red-400 rounded"></span> Unavailable
          </span>
          <span className="flex items-center gap-1">
            <span className="w-4 h-4 bg-gray-200 rounded"></span> No Response
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="min-w-full">
            <thead>
              <tr className="bg-gray-50">
                <th className="sticky left-0 z-10 bg-gray-50 px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider min-w-[200px] border-r">
                  User
                </th>
                {events.map((event) => (
                  <th
                    key={event.id}
                    className="px-2 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider min-w-[60px]"
                  >
                    <Link
                      to={`/events/${event.id}`}
                      className="hover:text-indigo-600 block"
                      title={event.title}
                    >
                      <div className="truncate max-w-[60px]">{formatDate(event.event_date)}</div>
                      <div className="truncate max-w-[60px] font-normal normal-case text-gray-400">
                        {event.title.substring(0, 8)}...
                      </div>
                    </Link>
                  </th>
                ))}
                <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider min-w-[80px] border-l">
                  Rate
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {rows.map((row) => (
                <tr key={row.user.user_id} className="hover:bg-gray-50">
                  <td className="sticky left-0 z-10 bg-white px-4 py-2 whitespace-nowrap border-r">
                    <div className="flex items-center">
                      <div className="h-8 w-8 rounded-full bg-indigo-100 flex items-center justify-center mr-3">
                        <span className="text-indigo-700 text-xs font-medium">
                          {row.user.username.substring(0, 2).toUpperCase()}
                        </span>
                      </div>
                      <div>
                        <p className="text-sm font-medium text-gray-900">{row.user.username}</p>
                        <p className="text-xs text-gray-500">
                          {row.user.events_checked_in}/{row.user.total_events} attended
                        </p>
                      </div>
                    </div>
                  </td>
                  {row.cells.map((cell, idx) => (
                    <td key={idx} className="px-1 py-2 text-center">
                      <span
                        className={`inline-flex items-center justify-center w-8 h-8 rounded text-xs font-medium ${getCellStyle(cell)}`}
                        title={cell.replace('_', ' ')}
                      >
                        {getCellLabel(cell)}
                      </span>
                    </td>
                  ))}
                  <td className="px-4 py-2 text-center border-l">
                    <div className="flex flex-col items-center">
                      <span
                        className={`text-sm font-bold ${
                          row.user.attendance_rate >= 70
                            ? 'text-green-600'
                            : row.user.attendance_rate >= 40
                            ? 'text-yellow-600'
                            : 'text-red-600'
                        }`}
                      >
                        {row.user.attendance_rate.toFixed(0)}%
                      </span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
            {/* Footer with event stats */}
            <tfoot className="bg-gray-50">
              <tr>
                <td className="sticky left-0 z-10 bg-gray-50 px-4 py-3 text-sm font-medium text-gray-900 border-r">
                  Event Totals
                </td>
                {events.map((event) => (
                  <td key={event.id} className="px-1 py-3 text-center">
                    <div className="text-xs">
                      <span className="text-green-600 font-medium">{event.total_checked_in}</span>
                      <span className="text-gray-400">/</span>
                      <span className="text-yellow-600">{event.total_available}</span>
                    </div>
                  </td>
                ))}
                <td className="px-4 py-3 border-l"></td>
              </tr>
            </tfoot>
          </table>
        </div>
      </div>
    </div>
  );
}
