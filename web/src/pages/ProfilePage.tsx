import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { ProfileService, MeritService, AwardService } from '../services/merit';
import type {
  ProfileResponse,
  MeritHistoryEntry,
  AwardResponse,
  AwardTier,
} from '../services/types';

// Award tier styling configuration
const tierConfig: Record<AwardTier, { bg: string; border: string; text: string; icon: string }> = {
  bronze: {
    bg: 'bg-amber-50',
    border: 'border-amber-300',
    text: 'text-amber-700',
    icon: '🥉',
  },
  silver: {
    bg: 'bg-gray-100',
    border: 'border-gray-400',
    text: 'text-gray-700',
    icon: '🥈',
  },
  gold: {
    bg: 'bg-yellow-50',
    border: 'border-yellow-400',
    text: 'text-yellow-700',
    icon: '🥇',
  },
};

// Award Card Component
function AwardCard({ award }: { award: AwardResponse }) {
  const config = tierConfig[award.tier];
  const awardedDate = new Date(award.awarded_at).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });

  return (
    <div
      className={`${config.bg} ${config.border} border-2 rounded-lg p-4 transition-transform hover:scale-105`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <h3 className={`font-semibold ${config.text}`}>{award.title}</h3>
          {award.description && (
            <p className="mt-1 text-sm text-gray-600">{award.description}</p>
          )}
        </div>
        <span className="text-2xl ml-2">{config.icon}</span>
      </div>
      <div className="mt-3 flex items-center justify-between">
        <span
          className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium capitalize ${config.bg} ${config.text} border ${config.border}`}
        >
          {award.tier}
        </span>
        <span className="text-xs text-gray-500">{awardedDate}</span>
      </div>
    </div>
  );
}

export default function ProfilePage() {
  const { username } = useParams<{ username: string }>();
  const { user } = useAuth();
  const [profile, setProfile] = useState<ProfileResponse | null>(null);
  const [meritHistory, setMeritHistory] = useState<MeritHistoryEntry[]>([]);
  const [awards, setAwards] = useState<AwardResponse[]>([]);
  const [historyPage, setHistoryPage] = useState(1);
  const [historyTotalPages, setHistoryTotalPages] = useState(1);
  const [isLoading, setIsLoading] = useState(true);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [awardsLoading, setAwardsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isOwnProfile = user?.username === username;
  const canSeeMerit = profile && (ProfileService.isPrivateProfile(profile) || ProfileService.isAdminProfile(profile));
  const isAdminView = profile && ProfileService.isAdminProfile(profile);

  useEffect(() => {
    const loadProfile = async () => {
      if (!username) return;
      
      setIsLoading(true);
      setError(null);
      
      try {
        const profileData = await ProfileService.getProfile(username);
        setProfile(profileData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load profile');
      } finally {
        setIsLoading(false);
      }
    };

    loadProfile();
  }, [username]);

  // Load awards (public - visible on all profiles)
  useEffect(() => {
    const loadAwards = async () => {
      if (!username) return;
      
      setAwardsLoading(true);
      try {
        const awardsData = await AwardService.getUserAwards(username);
        setAwards(awardsData.awards);
      } catch (err) {
        console.error('Failed to load awards:', err);
      } finally {
        setAwardsLoading(false);
      }
    };

    loadAwards();
  }, [username]);

  useEffect(() => {
    const loadHistory = async () => {
      if (!isOwnProfile || !canSeeMerit) return;
      
      setHistoryLoading(true);
      try {
        const historyData = await MeritService.getMyMeritHistory(historyPage, 10);
        setMeritHistory(historyData.history);
        setHistoryTotalPages(historyData.total_pages);
      } catch (err) {
        console.error('Failed to load merit history:', err);
      } finally {
        setHistoryLoading(false);
      }
    };

    loadHistory();
  }, [isOwnProfile, canSeeMerit, historyPage]);

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const formatDateTime = (dateString: string) => {
    return new Date(dateString).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (error || !profile) {
    return (
      <div className="min-h-screen bg-gray-50 py-12">
        <div className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="bg-white shadow rounded-lg p-6 text-center">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
              />
            </svg>
            <h2 className="mt-4 text-xl font-semibold text-gray-900">User Not Found</h2>
            <p className="mt-2 text-gray-600">{error || 'The user you are looking for does not exist.'}</p>
            <Link
              to="/"
              className="mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-indigo-600 bg-indigo-100 hover:bg-indigo-200"
            >
              Go back home
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Profile Header */}
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="bg-gradient-to-r from-indigo-500 to-purple-600 h-32"></div>
          <div className="px-6 pb-6">
            <div className="-mt-12 flex items-end space-x-5">
              <div className="flex-shrink-0">
                <div className="h-24 w-24 rounded-full bg-white ring-4 ring-white flex items-center justify-center text-3xl font-bold text-indigo-600 uppercase">
                  {profile.username.charAt(0)}
                </div>
              </div>
              <div className="mt-16 flex-1 min-w-0">
                <div className="flex items-center space-x-3">
                  <h1 className="text-2xl font-bold text-gray-900 truncate">
                    {profile.username}
                  </h1>
                  {isAdminView && (profile as { is_admin: boolean }).is_admin && (
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                      Admin
                    </span>
                  )}
                  {isOwnProfile && (
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                      You
                    </span>
                  )}
                </div>
                <p className="text-sm text-gray-500">
                  Member since {formatDate(profile.created_at)}
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Profile Details */}
        <div className="mt-6 grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Basic Info */}
          <div className="lg:col-span-2 bg-white shadow rounded-lg p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Profile Information</h2>
            <dl className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <dt className="text-sm font-medium text-gray-500">Username</dt>
                <dd className="mt-1 text-sm text-gray-900">{profile.username}</dd>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500">Year Joined</dt>
                <dd className="mt-1 text-sm text-gray-900">{profile.year_joined}</dd>
              </div>
              
              {/* Private/Admin fields */}
              {(ProfileService.isPrivateProfile(profile) || ProfileService.isAdminProfile(profile)) && (
                <>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">Email</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {'email' in profile ? profile.email : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">Registration Number</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {'reg_number' in profile ? profile.reg_number : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">Phone Number</dt>
                    <dd className="mt-1 text-sm text-gray-900">
                      {'phone_number' in profile ? profile.phone_number : 'N/A'}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-sm font-medium text-gray-500">Email Verified</dt>
                    <dd className="mt-1">
                      {'email_verified' in profile && profile.email_verified ? (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                          Verified
                        </span>
                      ) : (
                        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800">
                          Not Verified
                        </span>
                      )}
                    </dd>
                  </div>
                </>
              )}
            </dl>
          </div>

          {/* Merit Card - Only shown if user can see merit */}
          {canSeeMerit && (
            <div className="bg-white shadow rounded-lg p-6">
              <h2 className="text-lg font-semibold text-gray-900 mb-4">Merit Points</h2>
              <div className="text-center">
                <div className="text-5xl font-bold text-indigo-600">
                  {'merit_points' in profile ? profile.merit_points : 0}
                </div>
                <p className="mt-2 text-sm text-gray-500">
                  {isOwnProfile ? 'Your merit points' : 'Current merit points'}
                </p>
              </div>
              {isOwnProfile && (
                <p className="mt-4 text-xs text-gray-400 text-center">
                  Merit points are private and only visible to you and admins.
                </p>
              )}
            </div>
          )}

          {/* Public view note */}
          {!canSeeMerit && !isOwnProfile && (
            <div className="bg-gray-100 rounded-lg p-6 flex items-center justify-center">
              <p className="text-sm text-gray-500 text-center">
                Merit points are private and only visible to the user themselves and admins.
              </p>
            </div>
          )}
        </div>

        {/* Merit History - Only for own profile */}
        {isOwnProfile && canSeeMerit && (
          <div className="mt-6 bg-white shadow rounded-lg p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Merit History</h2>
            
            {historyLoading ? (
              <div className="flex justify-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              </div>
            ) : meritHistory.length === 0 ? (
              <p className="text-center text-gray-500 py-8">No merit changes yet.</p>
            ) : (
              <>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead>
                      <tr>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Date
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Change
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          New Total
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Reason
                        </th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          By
                        </th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200">
                      {meritHistory.map((entry) => (
                        <tr key={entry.id}>
                          <td className="px-4 py-3 text-sm text-gray-500 whitespace-nowrap">
                            {formatDateTime(entry.created_at)}
                          </td>
                          <td className="px-4 py-3 text-sm whitespace-nowrap">
                            <span
                              className={`inline-flex items-center px-2 py-0.5 rounded text-sm font-medium ${
                                entry.change_amount > 0
                                  ? 'bg-green-100 text-green-800'
                                  : 'bg-red-100 text-red-800'
                              }`}
                            >
                              {entry.change_amount > 0 ? '+' : ''}
                              {entry.change_amount}
                            </span>
                          </td>
                          <td className="px-4 py-3 text-sm text-gray-900 whitespace-nowrap">
                            {entry.new_total}
                          </td>
                          <td className="px-4 py-3 text-sm text-gray-500">
                            {entry.reason}
                          </td>
                          <td className="px-4 py-3 text-sm text-gray-500 whitespace-nowrap">
                            {entry.admin_username || 'System'}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                {/* Pagination */}
                {historyTotalPages > 1 && (
                  <div className="mt-4 flex items-center justify-between">
                    <button
                      onClick={() => setHistoryPage((p) => Math.max(1, p - 1))}
                      disabled={historyPage === 1}
                      className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                    >
                      Previous
                    </button>
                    <span className="text-sm text-gray-500">
                      Page {historyPage} of {historyTotalPages}
                    </span>
                    <button
                      onClick={() => setHistoryPage((p) => Math.min(historyTotalPages, p + 1))}
                      disabled={historyPage === historyTotalPages}
                      className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                    >
                      Next
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        )}

        {/* Awards Section - Public, visible on all profiles */}
        <div className="mt-6 bg-white shadow rounded-lg p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            🏆 Awards & Achievements
          </h2>
          
          {awardsLoading ? (
            <div className="flex justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
            </div>
          ) : awards.length === 0 ? (
            <p className="text-center text-gray-500 py-8">
              {isOwnProfile ? "You haven't received any awards yet." : "No awards yet."}
            </p>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {awards.map((award) => (
                <AwardCard key={award.id} award={award} />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
