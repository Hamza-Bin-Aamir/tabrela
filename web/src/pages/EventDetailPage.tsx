import { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { AttendanceService } from '../services/attendance';
import { AdminService } from '../services/admin';
import { useAuth } from '../context/AuthContext';
import type { Event, AttendanceRecord, EventType } from '../services/types';

export default function EventDetailPage() {
  const { eventId } = useParams<{ eventId: string }>();
  const navigate = useNavigate();
  const { user } = useAuth();

  const [event, setEvent] = useState<Event | null>(null);
  const [attendance, setAttendance] = useState<AttendanceRecord[]>([]);
  const [myAttendance, setMyAttendance] = useState<AttendanceRecord | null>(null);
  const [stats, setStats] = useState({ total_available: 0, total_checked_in: 0 });
  const [isAdmin, setIsAdmin] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    const checkAdmin = async () => {
      try {
        const response = await AdminService.checkAdminStatus();
        setIsAdmin(response.is_admin);
      } catch {
        setIsAdmin(false);
      }
    };
    checkAdmin();
  }, []);

  const loadEventData = useCallback(async () => {
    if (!eventId) return;

    setIsLoading(true);
    setError(null);

    try {
      const response = await AttendanceService.getEventAttendance(eventId);
      setEvent(response.event);
      setAttendance(response.attendance);
      setStats({
        total_available: response.stats.total_available,
        total_checked_in: response.stats.total_checked_in,
      });

      // Find current user's attendance
      const myRecord = response.attendance.find((a) => a.user_id === user?.id);
      setMyAttendance(myRecord || null);
    } catch (err) {
      setError('Failed to load event. Please try again.');
      console.error('Failed to load event:', err);
    } finally {
      setIsLoading(false);
    }
  }, [eventId, user?.id]);

  useEffect(() => {
    loadEventData();
  }, [loadEventData]);

  const handleSetAvailability = async (isAvailable: boolean) => {
    if (!eventId) return;

    setActionLoading('availability');
    setError(null);
    setSuccessMessage(null);

    try {
      await AttendanceService.setAvailability(eventId, isAvailable);
      setSuccessMessage(isAvailable ? 'Marked as available!' : 'Marked as unavailable.');
      await loadEventData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update availability');
    } finally {
      setActionLoading(null);
    }
  };

  const handleCheckIn = async (userId: string, isCheckedIn: boolean) => {
    if (!eventId) return;

    setActionLoading(userId);
    setError(null);
    setSuccessMessage(null);

    try {
      await AttendanceService.checkInUser(eventId, userId, isCheckedIn);
      setSuccessMessage(isCheckedIn ? 'User checked in!' : 'Check-in revoked.');
      await loadEventData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update check-in status');
    } finally {
      setActionLoading(null);
    }
  };

  const handleRevokeAvailability = async (userId: string) => {
    if (!eventId) return;

    if (!confirm('Are you sure you want to revoke this user\'s availability?')) {
      return;
    }

    setActionLoading(userId);
    setError(null);
    setSuccessMessage(null);

    try {
      await AttendanceService.revokeAvailability(eventId, userId);
      setSuccessMessage('Availability revoked.');
      await loadEventData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to revoke availability');
    } finally {
      setActionLoading(null);
    }
  };

  const handleLockEvent = async (isLocked: boolean) => {
    if (!eventId) return;

    setActionLoading('lock');
    setError(null);
    setSuccessMessage(null);

    try {
      await AttendanceService.lockEvent(eventId, isLocked);
      setSuccessMessage(isLocked ? 'Event locked!' : 'Event unlocked.');
      await loadEventData();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update lock status');
    } finally {
      setActionLoading(null);
    }
  };

  const handleDeleteEvent = async () => {
    if (!eventId) return;

    if (!confirm('Are you sure you want to delete this event? This cannot be undone.')) {
      return;
    }

    setActionLoading('delete');
    setError(null);

    try {
      await AttendanceService.deleteEvent(eventId);
      navigate('/events');
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to delete event');
      setActionLoading(null);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getEventTypeBadge = (eventType: EventType) => {
    const badges: Record<EventType, { bg: string; text: string; label: string }> = {
      tournament: { bg: 'bg-purple-100', text: 'text-purple-800', label: 'Tournament' },
      weekly_match: { bg: 'bg-blue-100', text: 'text-blue-800', label: 'Weekly Match' },
      meeting: { bg: 'bg-green-100', text: 'text-green-800', label: 'Meeting' },
      other: { bg: 'bg-gray-100', text: 'text-gray-800', label: 'Other' },
    };
    const badge = badges[eventType] || badges.other;
    return (
      <span className={`px-3 py-1 text-sm font-medium rounded-full ${badge.bg} ${badge.text}`}>
        {badge.label}
      </span>
    );
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

  if (!event) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-bold text-gray-800 mb-2">Event not found</h1>
          <Link to="/events" className="text-indigo-600 hover:text-indigo-800">
            ← Back to events
          </Link>
        </div>
      </div>
    );
  }

  const isAvailable = myAttendance?.is_available ?? false;
  const isCheckedIn = myAttendance?.is_checked_in ?? false;

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Back link */}
        <Link to="/events" className="text-indigo-600 hover:text-indigo-800 mb-4 inline-block">
          ← Back to events
        </Link>

        {/* Messages */}
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-md">
            {error}
          </div>
        )}
        {successMessage && (
          <div className="mb-4 p-4 bg-green-50 border border-green-200 text-green-700 rounded-md">
            {successMessage}
          </div>
        )}

        {/* Event Header */}
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <div className="flex flex-wrap justify-between items-start gap-4 mb-4">
            <div className="flex items-center gap-3">
              {getEventTypeBadge(event.event_type)}
              {event.is_locked && (
                <span className="px-3 py-1 text-sm font-medium rounded-full bg-red-100 text-red-800">
                  🔒 Locked
                </span>
              )}
            </div>
            {isAdmin && (
              <div className="flex gap-2">
                <Link
                  to={`/events/${event.id}/edit`}
                  className="px-3 py-1 text-sm font-medium rounded-md border border-gray-300 text-gray-700 hover:bg-gray-50"
                >
                  Edit
                </Link>
                <button
                  onClick={() => handleLockEvent(!event.is_locked)}
                  disabled={actionLoading === 'lock'}
                  className={`px-3 py-1 text-sm font-medium rounded-md ${
                    event.is_locked
                      ? 'border border-green-300 text-green-700 hover:bg-green-50'
                      : 'border border-yellow-300 text-yellow-700 hover:bg-yellow-50'
                  }`}
                >
                  {actionLoading === 'lock' ? '...' : event.is_locked ? 'Unlock' : 'Lock'}
                </button>
                <button
                  onClick={handleDeleteEvent}
                  disabled={actionLoading === 'delete'}
                  className="px-3 py-1 text-sm font-medium rounded-md border border-red-300 text-red-700 hover:bg-red-50"
                >
                  {actionLoading === 'delete' ? '...' : 'Delete'}
                </button>
              </div>
            )}
          </div>

          <h1 className="text-3xl font-bold text-gray-900 mb-4">{event.title}</h1>

          {event.description && (
            <p className="text-gray-600 mb-4">{event.description}</p>
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm text-gray-600">
            <div className="flex items-center">
              <svg className="w-5 h-5 mr-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              {formatDate(event.event_date)}
            </div>
            {event.location && (
              <div className="flex items-center">
                <svg className="w-5 h-5 mr-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
                {event.location}
              </div>
            )}
          </div>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-2 gap-4 mb-6">
          <div className="bg-white rounded-lg shadow p-4">
            <h3 className="text-sm font-medium text-gray-500">Available</h3>
            <p className="mt-1 text-2xl font-bold text-green-600">{stats.total_available}</p>
          </div>
          <div className="bg-white rounded-lg shadow p-4">
            <h3 className="text-sm font-medium text-gray-500">Checked In</h3>
            <p className="mt-1 text-2xl font-bold text-blue-600">{stats.total_checked_in}</p>
          </div>
        </div>

        {/* My Availability */}
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Your Status</h2>
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-2">
              <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                isAvailable ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
              }`}>
                {isAvailable ? '✓ Available' : '✗ Unavailable'}
              </span>
              {isCheckedIn && (
                <span className="px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
                  ✓ Checked In
                </span>
              )}
            </div>
            {!event.is_locked && (
              <div className="flex gap-2">
                <button
                  onClick={() => handleSetAvailability(true)}
                  disabled={actionLoading === 'availability' || isAvailable}
                  className={`px-4 py-2 text-sm font-medium rounded-md ${
                    isAvailable
                      ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                      : 'bg-green-600 text-white hover:bg-green-700'
                  }`}
                >
                  {actionLoading === 'availability' ? '...' : 'Mark Available'}
                </button>
                <button
                  onClick={() => handleSetAvailability(false)}
                  disabled={actionLoading === 'availability' || !isAvailable}
                  className={`px-4 py-2 text-sm font-medium rounded-md ${
                    !isAvailable
                      ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                      : 'bg-gray-600 text-white hover:bg-gray-700'
                  }`}
                >
                  {actionLoading === 'availability' ? '...' : 'Mark Unavailable'}
                </button>
              </div>
            )}
            {event.is_locked && (
              <p className="text-sm text-gray-500">Event is locked - attendance cannot be changed.</p>
            )}
          </div>
        </div>

        {/* Attendance List */}
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">Attendance ({attendance.length})</h2>
          </div>

          {attendance.length === 0 ? (
            <div className="p-6 text-center text-gray-500">
              No one has marked their availability yet.
            </div>
          ) : (
            <div className="divide-y divide-gray-200">
              {attendance.map((record) => {
                const initials = record.username.substring(0, 2);
                
                return (
                  <div key={record.id} className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="h-10 w-10 rounded-full bg-indigo-100 flex items-center justify-center">
                        <span className="text-indigo-700 font-medium">
                          {initials.toUpperCase()}
                        </span>
                      </div>
                      <div>
                        <p className="text-sm font-medium text-gray-900">
                          {record.username}
                          {record.user_id === user?.id && (
                            <span className="ml-2 text-xs text-indigo-600">(You)</span>
                          )}
                        </p>
                        <div className="flex items-center gap-2 mt-1">
                          <span className={`text-xs px-2 py-0.5 rounded-full ${
                            record.is_available
                              ? 'bg-green-100 text-green-700'
                              : 'bg-gray-100 text-gray-700'
                          }`}>
                            {record.is_available ? 'Available' : 'Unavailable'}
                          </span>
                          {record.is_checked_in && (
                            <span className="text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-700">
                              Checked In
                            </span>
                          )}
                        </div>
                      </div>
                    </div>

                    {isAdmin && !event.is_locked && (
                      <div className="flex gap-2">
                        <button
                          onClick={() => handleCheckIn(record.user_id, !record.is_checked_in)}
                          disabled={actionLoading === record.user_id}
                          className={`px-3 py-1 text-xs font-medium rounded-md ${
                            record.is_checked_in
                              ? 'border border-gray-300 text-gray-700 hover:bg-gray-50'
                              : 'bg-blue-600 text-white hover:bg-blue-700'
                          }`}
                        >
                          {actionLoading === record.user_id
                            ? '...'
                            : record.is_checked_in
                            ? 'Undo Check-in'
                            : 'Check In'}
                        </button>
                        <button
                          onClick={() => handleRevokeAvailability(record.user_id)}
                          disabled={actionLoading === record.user_id}
                          className="px-3 py-1 text-xs font-medium rounded-md border border-red-300 text-red-700 hover:bg-red-50"
                        >
                          {actionLoading === record.user_id ? '...' : 'Revoke'}
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
