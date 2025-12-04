import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { AdminService } from '../services/admin';
import { MeritService } from '../services/merit';
import { AttendanceService } from '../services/attendance';
import { TabulationService } from '../services/tabulation';
import type {
  MeritResponse,
  Event,
  EventType,
  AllocationRole,
} from '../services/types';

// Quick Links configurations based on user state
interface QuickLink {
  to: string;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
}

// Icon components
const LoginIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
  </svg>
);

const SignupIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
  </svg>
);

const EventsIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
  </svg>
);

const ProfileIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
  </svg>
);

const AboutIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

const AttendanceIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
  </svg>
);

const MeritIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
  </svg>
);

const AdminIcon = () => (
  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
  </svg>
);

// User allocation info for an event
interface UserAllocation {
  matchId: string;
  room: string | null;
  motion: string | null;
  role: AllocationRole;
  positions: string[];
  seriesName: string;
}

// Attendance status type
type AttendanceStatus = 'no_response' | 'available' | 'unavailable' | 'checked_in';

export default function Home() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const [isAdmin, setIsAdmin] = useState(false);
  const [adminLoading, setAdminLoading] = useState(false);
  const [merit, setMerit] = useState<MeritResponse | null>(null);
  const [meritLoading, setMeritLoading] = useState(false);
  const [upcomingEvent, setUpcomingEvent] = useState<Event | null>(null);
  const [attendanceStatus, setAttendanceStatus] = useState<AttendanceStatus>('no_response');
  const [userAllocation, setUserAllocation] = useState<UserAllocation | null>(null);
  const [eventLoading, setEventLoading] = useState(false);

  // Check admin status
  useEffect(() => {
    if (!isAuthenticated) {
      setIsAdmin(false);
      return;
    }

    const checkAdmin = async () => {
      setAdminLoading(true);
      try {
        const response = await AdminService.checkAdminStatus();
        setIsAdmin(response.is_admin);
      } catch {
        setIsAdmin(false);
      } finally {
        setAdminLoading(false);
      }
    };
    checkAdmin();
  }, [isAuthenticated]);

  // Load merit points
  useEffect(() => {
    if (!isAuthenticated) {
      setMerit(null);
      return;
    }

    const loadMerit = async () => {
      setMeritLoading(true);
      try {
        const meritData = await MeritService.getMyMerit();
        setMerit(meritData);
      } catch (err) {
        console.error('Failed to load merit:', err);
      } finally {
        setMeritLoading(false);
      }
    };
    loadMerit();
  }, [isAuthenticated]);

  // Load upcoming event and user's attendance/allocation
  useEffect(() => {
    if (!isAuthenticated || !user) {
      setUpcomingEvent(null);
      setAttendanceStatus('no_response');
      setUserAllocation(null);
      return;
    }

    const loadUpcomingEvent = async () => {
      setEventLoading(true);
      try {
        // Get upcoming events
        const eventsResponse = await AttendanceService.listEvents(1, 1, undefined, true);
        
        if (eventsResponse.events.length === 0) {
          setUpcomingEvent(null);
          setEventLoading(false);
          return;
        }

        const event = eventsResponse.events[0];
        setUpcomingEvent(event);

        // Get user's attendance for this event
        try {
          const attendance = await AttendanceService.getMyAttendance(event.id);
          if (attendance.is_checked_in) {
            setAttendanceStatus('checked_in');
          } else if (attendance.is_available) {
            setAttendanceStatus('available');
          } else {
            setAttendanceStatus('unavailable');
          }
        } catch {
          // No attendance record means no response
          setAttendanceStatus('no_response');
        }

        // Check for matches associated with this event
        try {
          const matchesResponse = await TabulationService.listMatches({
            eventId: event.id,
            page: 1,
            perPage: 50,
          });

          // Find user's allocation in any match
          for (const match of matchesResponse.matches) {
            // Check adjudicators
            const adjAllocation = match.adjudicators.find(
              (adj) => adj.user_id === user.id
            );
            if (adjAllocation) {
              const role = adjAllocation.is_voting ? 'voting_adjudicator' : 'non_voting_adjudicator';
              const positions = [adjAllocation.is_chair ? 'Chair' : (adjAllocation.is_voting ? 'Panelist' : 'Trainee')];
              setUserAllocation({
                matchId: match.id,
                room: match.room_name,
                motion: match.motion,
                role,
                positions,
                seriesName: match.series_name,
              });
              break;
            }

            // Check teams for speakers/resources
            for (const team of match.teams) {
              const speakerAllocation = team.speakers.find(
                (s) => s.user_id === user.id
              );
              if (speakerAllocation) {
                const positions: string[] = [];
                if (speakerAllocation.two_team_speaker_role) {
                  positions.push(formatSpeakerRole(speakerAllocation.two_team_speaker_role));
                }
                if (speakerAllocation.four_team_speaker_role) {
                  positions.push(formatSpeakerRole(speakerAllocation.four_team_speaker_role));
                }
                const teamPosition = team.two_team_position || team.four_team_position;
                if (teamPosition) {
                  positions.push(formatTeamPosition(teamPosition));
                }
                setUserAllocation({
                  matchId: match.id,
                  room: match.room_name,
                  motion: match.motion,
                  role: 'speaker',
                  positions,
                  seriesName: match.series_name,
                });
                break;
              }

              const resourceAllocation = team.resources.find(
                (r) => r.user_id === user.id
              );
              if (resourceAllocation) {
                const teamPosition = team.two_team_position || team.four_team_position;
                setUserAllocation({
                  matchId: match.id,
                  room: match.room_name,
                  motion: match.motion,
                  role: 'resource',
                  positions: teamPosition ? [`Resource - ${formatTeamPosition(teamPosition)}`] : ['Resource'],
                  seriesName: match.series_name,
                });
                break;
              }
            }
            if (userAllocation) break;
          }
        } catch (err) {
          console.error('Failed to load matches:', err);
        }
      } catch (err) {
        console.error('Failed to load upcoming event:', err);
      } finally {
        setEventLoading(false);
      }
    };

    loadUpcomingEvent();
  }, [isAuthenticated, user]);

  // Format speaker role for display
  const formatSpeakerRole = (role: string): string => {
    return role
      .split('_')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  // Format team position for display
  const formatTeamPosition = (position: string): string => {
    return position
      .split('_')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  // Build quick links based on auth state
  const getQuickLinks = (): QuickLink[] => {
    if (!isAuthenticated) {
      return [
        {
          to: '/login',
          label: 'Log In',
          description: 'Access your account',
          icon: <LoginIcon />,
          color: 'bg-blue-500 hover:bg-blue-600',
        },
        {
          to: '/signup',
          label: 'Sign Up',
          description: 'Create a new account',
          icon: <SignupIcon />,
          color: 'bg-green-500 hover:bg-green-600',
        },
        {
          to: '/about',
          label: 'About',
          description: 'Learn about Tabrela',
          icon: <AboutIcon />,
          color: 'bg-purple-500 hover:bg-purple-600',
        },
      ];
    }

    const links: QuickLink[] = [
      {
        to: '/events',
        label: 'Events',
        description: 'View upcoming events',
        icon: <EventsIcon />,
        color: 'bg-blue-500 hover:bg-blue-600',
      },
      {
        to: `/users/${user?.username}`,
        label: 'My Profile',
        description: 'View your profile',
        icon: <ProfileIcon />,
        color: 'bg-green-500 hover:bg-green-600',
      },
      {
        to: '/about',
        label: 'About',
        description: 'Learn about Tabrela',
        icon: <AboutIcon />,
        color: 'bg-purple-500 hover:bg-purple-600',
      },
    ];

    if (isAdmin) {
      links.push(
        {
          to: '/attendance/dashboard',
          label: 'Attendance',
          description: 'Manage attendance',
          icon: <AttendanceIcon />,
          color: 'bg-amber-500 hover:bg-amber-600',
        },
        {
          to: '/admin/merit',
          label: 'Merit',
          description: 'Manage merit points',
          icon: <MeritIcon />,
          color: 'bg-rose-500 hover:bg-rose-600',
        },
        {
          to: '/admin',
          label: 'Admin',
          description: 'Admin dashboard',
          icon: <AdminIcon />,
          color: 'bg-gray-700 hover:bg-gray-800',
        }
      );
    }

    return links;
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
      <span className={`px-2 py-1 text-xs font-medium rounded-full ${badge.bg} ${badge.text}`}>
        {badge.label}
      </span>
    );
  };

  const getAttendanceStatusBadge = () => {
    const statusConfig = {
      no_response: { bg: 'bg-gray-100', text: 'text-gray-700', label: 'No Response', icon: '⚪' },
      available: { bg: 'bg-green-100', text: 'text-green-700', label: 'Available', icon: '✅' },
      unavailable: { bg: 'bg-red-100', text: 'text-red-700', label: 'Unavailable', icon: '❌' },
      checked_in: { bg: 'bg-blue-100', text: 'text-blue-700', label: 'Checked In', icon: '📍' },
    };
    const config = statusConfig[attendanceStatus];
    return (
      <span className={`inline-flex items-center gap-1 px-3 py-1 text-sm font-medium rounded-full ${config.bg} ${config.text}`}>
        <span>{config.icon}</span>
        {config.label}
      </span>
    );
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

  const quickLinks = getQuickLinks();
  const isLoadingAny = authLoading || adminLoading;

  return (
    <div className="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-purple-50 py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Welcome Header */}
        <div className="text-center mb-10">
          <h1 className="text-4xl font-extrabold text-gray-900 sm:text-5xl">
            Welcome to <span className="text-indigo-600">Tabrela</span>
          </h1>
          {isAuthenticated && user && (
            <p className="mt-3 text-xl text-gray-600">
              Hello, <span className="font-semibold text-indigo-600">{user.username}</span>! 👋
            </p>
          )}
          <p className="mt-2 text-gray-500">
            {isAuthenticated
              ? 'Your debating hub for events, matches, and merit tracking'
              : 'Join us to manage events, track attendance, and more'}
          </p>
        </div>

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Quick Links Card */}
          <div className="lg:col-span-2">
            <div className="bg-white rounded-2xl shadow-lg p-6 h-full">
              <h2 className="text-xl font-bold text-gray-900 mb-4 flex items-center gap-2">
                <svg className="w-6 h-6 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
                Quick Links
              </h2>
              {isLoadingAny ? (
                <div className="flex items-center justify-center h-32">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                </div>
              ) : (
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-4">
                  {quickLinks.map((link) => (
                    <Link
                      key={link.to}
                      to={link.to}
                      className={`${link.color} text-white rounded-xl p-4 transition-all duration-200 transform hover:scale-105 hover:shadow-lg group`}
                    >
                      <div className="flex flex-col items-center text-center">
                        <div className="p-2 bg-white/20 rounded-lg group-hover:bg-white/30 transition-colors">
                          {link.icon}
                        </div>
                        <h3 className="mt-2 font-semibold">{link.label}</h3>
                        <p className="text-xs text-white/80 mt-1">{link.description}</p>
                      </div>
                    </Link>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Merit Points Card */}
          <div className="lg:col-span-1">
            <div className="bg-white rounded-2xl shadow-lg p-6 h-full">
              <h2 className="text-xl font-bold text-gray-900 mb-4 flex items-center gap-2">
                <MeritIcon />
                <span className="text-indigo-600">Merit Points</span>
              </h2>
              {!isAuthenticated ? (
                <div className="text-center py-6">
                  <div className="text-5xl mb-3">🔒</div>
                  <p className="text-gray-500 mb-3">Log in to see your merit points</p>
                  <Link
                    to="/login"
                    className="inline-flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
                  >
                    Log In
                  </Link>
                </div>
              ) : meritLoading ? (
                <div className="flex items-center justify-center h-32">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
                </div>
              ) : merit ? (
                <div className="text-center py-4">
                  <div className="relative inline-block">
                    <div className="absolute inset-0 bg-gradient-to-r from-yellow-400 via-amber-500 to-orange-500 rounded-full blur-lg opacity-30 animate-pulse"></div>
                    <div className="relative bg-gradient-to-br from-yellow-100 to-amber-100 rounded-full p-6 border-4 border-amber-300">
                      <span className="text-4xl font-extrabold text-amber-700">
                        {merit.merit_points}
                      </span>
                    </div>
                  </div>
                  <p className="mt-4 text-gray-600">
                    {merit.merit_points === 0
                      ? 'Start earning points by participating!'
                      : merit.merit_points < 50
                      ? 'Keep going! You\'re making progress!'
                      : merit.merit_points < 100
                      ? 'Great work! You\'re doing fantastic!'
                      : 'Outstanding! You\'re a star contributor! ⭐'}
                  </p>
                  <Link
                    to={`/users/${user?.username}`}
                    className="mt-4 inline-flex items-center text-sm text-indigo-600 hover:text-indigo-700"
                  >
                    View merit history →
                  </Link>
                </div>
              ) : (
                <div className="text-center py-6">
                  <div className="text-4xl mb-3">😕</div>
                  <p className="text-gray-500">Unable to load merit points</p>
                </div>
              )}
            </div>
          </div>

          {/* Upcoming Event Card */}
          <div className="lg:col-span-3">
            <div className="bg-white rounded-2xl shadow-lg p-6">
              <h2 className="text-xl font-bold text-gray-900 mb-4 flex items-center gap-2">
                <EventsIcon />
                <span className="text-indigo-600">Upcoming Event</span>
              </h2>
              {!isAuthenticated ? (
                <div className="text-center py-8">
                  <div className="text-5xl mb-3">🔒</div>
                  <p className="text-gray-500 mb-3">Log in to see upcoming events</p>
                  <Link
                    to="/login"
                    className="inline-flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
                  >
                    Log In
                  </Link>
                </div>
              ) : eventLoading ? (
                <div className="flex items-center justify-center h-40">
                  <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600"></div>
                </div>
              ) : upcomingEvent ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {/* Event Info */}
                  <div className="space-y-4">
                    <div className="flex items-start justify-between">
                      <div>
                        <div className="flex items-center gap-2 mb-2">
                          {getEventTypeBadge(upcomingEvent.event_type)}
                          {upcomingEvent.is_locked && (
                            <span className="px-2 py-1 text-xs font-medium rounded-full bg-red-100 text-red-700">
                              🔒 Locked
                            </span>
                          )}
                        </div>
                        <h3 className="text-2xl font-bold text-gray-900">{upcomingEvent.title}</h3>
                      </div>
                    </div>
                    
                    <div className="space-y-2 text-gray-600">
                      <div className="flex items-center gap-2">
                        <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                        <span>{formatDate(upcomingEvent.event_date)}</span>
                      </div>
                      {upcomingEvent.location && (
                        <div className="flex items-center gap-2">
                          <svg className="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
                          </svg>
                          <span>{upcomingEvent.location}</span>
                        </div>
                      )}
                      {upcomingEvent.description && (
                        <p className="text-sm text-gray-500 mt-2">{upcomingEvent.description}</p>
                      )}
                    </div>

                    <div className="pt-2">
                      <p className="text-sm font-medium text-gray-700 mb-2">Your Status:</p>
                      {getAttendanceStatusBadge()}
                    </div>

                    <Link
                      to={`/events/${upcomingEvent.id}`}
                      className="inline-flex items-center px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
                    >
                      View Event Details
                      <svg className="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                      </svg>
                    </Link>
                  </div>

                  {/* Match Allocation Info */}
                  <div className="bg-gradient-to-br from-indigo-50 to-purple-50 rounded-xl p-5 border border-indigo-100">
                    <h4 className="font-semibold text-gray-900 mb-3 flex items-center gap-2">
                      <svg className="w-5 h-5 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z" />
                      </svg>
                      Your Match Assignment
                    </h4>
                    {userAllocation ? (
                      <div className="space-y-3">
                        <div>
                          <p className="text-xs text-gray-500 uppercase tracking-wide">Round</p>
                          <p className="font-medium text-gray-900">{userAllocation.seriesName}</p>
                        </div>
                        <div>
                          <p className="text-xs text-gray-500 uppercase tracking-wide">Position(s)</p>
                          <div className="flex flex-wrap gap-2 mt-1">
                            {userAllocation.positions.map((pos, idx) => (
                              <span
                                key={idx}
                                className="px-2 py-1 text-sm bg-indigo-100 text-indigo-700 rounded-lg font-medium"
                              >
                                {pos}
                              </span>
                            ))}
                          </div>
                        </div>
                        {userAllocation.room && (
                          <div>
                            <p className="text-xs text-gray-500 uppercase tracking-wide">Room</p>
                            <p className="font-medium text-gray-900 flex items-center gap-1">
                              <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                              </svg>
                              {userAllocation.room}
                            </p>
                          </div>
                        )}
                        {userAllocation.motion && (
                          <div>
                            <p className="text-xs text-gray-500 uppercase tracking-wide">Motion</p>
                            <p className="text-sm text-gray-700 italic bg-white/50 p-2 rounded-lg">
                              "{userAllocation.motion}"
                            </p>
                          </div>
                        )}
                        <Link
                          to={`/events/${upcomingEvent.id}`}
                          className="inline-flex items-center text-sm text-indigo-600 hover:text-indigo-700 font-medium"
                        >
                          View Event Details →
                        </Link>
                      </div>
                    ) : (
                      <div className="text-center py-4">
                        <div className="text-3xl mb-2">📋</div>
                        <p className="text-gray-500 text-sm">
                          {attendanceStatus === 'available' || attendanceStatus === 'checked_in'
                            ? 'No match assignment yet. Check back later!'
                            : 'Mark yourself as available to get assigned to a match.'}
                        </p>
                      </div>
                    )}
                  </div>
                </div>
              ) : (
                <div className="text-center py-8">
                  <div className="text-5xl mb-3">📅</div>
                  <p className="text-gray-500 mb-3">No upcoming events scheduled</p>
                  <Link
                    to="/events"
                    className="inline-flex items-center text-indigo-600 hover:text-indigo-700"
                  >
                    View all events →
                  </Link>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer CTA for non-authenticated users */}
        {!isAuthenticated && (
          <div className="mt-10 text-center">
            <div className="bg-gradient-to-r from-indigo-600 to-purple-600 rounded-2xl p-8 text-white">
              <h2 className="text-2xl font-bold mb-3">Ready to Join?</h2>
              <p className="text-indigo-100 mb-6">
                Create an account to start tracking your debate journey, earn merit points, and connect with fellow debaters.
              </p>
              <div className="flex justify-center gap-4">
                <Link
                  to="/signup"
                  className="px-6 py-3 bg-white text-indigo-600 font-semibold rounded-lg hover:bg-indigo-50 transition-colors"
                >
                  Get Started
                </Link>
                <Link
                  to="/about"
                  className="px-6 py-3 bg-indigo-500 text-white font-semibold rounded-lg hover:bg-indigo-400 transition-colors"
                >
                  Learn More
                </Link>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
