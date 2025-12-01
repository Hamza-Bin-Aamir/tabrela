import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { AdminAwardService, AdminMeritService } from '../services/merit';
import type { AwardWithAdmin, UserMeritInfo, AwardTier } from '../services/types';

// Tier configuration
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

export default function AdminAwardsPage() {
  const [awards, setAwards] = useState<AwardWithAdmin[]>([]);
  const [users, setUsers] = useState<UserMeritInfo[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Create award modal state
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedUserId, setSelectedUserId] = useState<string>('');
  const [awardTitle, setAwardTitle] = useState('');
  const [awardDescription, setAwardDescription] = useState('');
  const [awardTier, setAwardTier] = useState<AwardTier>('bronze');
  const [awardReason, setAwardReason] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  // Edit modal state
  const [editAward, setEditAward] = useState<AwardWithAdmin | null>(null);
  const [editTitle, setEditTitle] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editTier, setEditTier] = useState<AwardTier>('bronze');
  const [isEditing, setIsEditing] = useState(false);

  // Upgrade modal state
  const [upgradeAward, setUpgradeAward] = useState<AwardWithAdmin | null>(null);
  const [upgradeTitle, setUpgradeTitle] = useState('');
  const [upgradeTier, setUpgradeTier] = useState<AwardTier>('silver');
  const [upgradeReason, setUpgradeReason] = useState('');
  const [isUpgrading, setIsUpgrading] = useState(false);

  const perPage = 20;

  const loadAwards = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await AdminAwardService.listAllAwards(currentPage, perPage);
      setAwards(response.awards);
      setTotalPages(response.total_pages);
      setTotal(response.total);
    } catch (err) {
      setError('Failed to load awards. Please try again.');
      console.error('Failed to load awards:', err);
    } finally {
      setIsLoading(false);
    }
  }, [currentPage]);

  const loadUsers = useCallback(async () => {
    try {
      // Load all users for the dropdown
      const response = await AdminMeritService.listAllMerits(1, 1000);
      setUsers(response.users);
    } catch (err) {
      console.error('Failed to load users:', err);
    }
  }, []);

  useEffect(() => {
    loadAwards();
    loadUsers();
  }, [loadAwards, loadUsers]);

  const handleCreateAward = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedUserId || !awardTitle.trim() || !awardReason.trim()) return;

    setIsCreating(true);
    setError(null);

    try {
      await AdminAwardService.createAward({
        user_id: selectedUserId,
        title: awardTitle.trim(),
        description: awardDescription.trim() || undefined,
        tier: awardTier,
        reason: awardReason.trim(),
      });
      setSuccessMessage('Award created successfully!');
      setShowCreateModal(false);
      resetCreateForm();
      loadAwards();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create award');
    } finally {
      setIsCreating(false);
    }
  };

  const handleUpgradeAward = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!upgradeAward || !upgradeReason.trim()) return;

    setIsUpgrading(true);
    setError(null);

    try {
      await AdminAwardService.upgradeAward(upgradeAward.id, {
        new_tier: upgradeTier,
        new_title: upgradeTitle !== upgradeAward.title ? upgradeTitle : undefined,
        reason: upgradeReason.trim(),
      });
      setSuccessMessage('Award upgraded successfully!');
      setUpgradeAward(null);
      resetUpgradeForm();
      loadAwards();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to upgrade award');
    } finally {
      setIsUpgrading(false);
    }
  };

  const handleEditAward = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editAward || !editTitle.trim()) return;

    setIsEditing(true);
    setError(null);

    try {
      await AdminAwardService.editAward(editAward.id, {
        title: editTitle.trim(),
        description: editDescription.trim() || undefined,
        tier: editTier,
      });
      setSuccessMessage('Award updated successfully!');
      setEditAward(null);
      resetEditForm();
      loadAwards();
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update award');
    } finally {
      setIsEditing(false);
    }
  };

  const resetCreateForm = () => {
    setSelectedUserId('');
    setAwardTitle('');
    setAwardDescription('');
    setAwardTier('bronze');
    setAwardReason('');
  };

  const resetEditForm = () => {
    setEditTitle('');
    setEditDescription('');
    setEditTier('bronze');
  };

  const resetUpgradeForm = () => {
    setUpgradeTitle('');
    setUpgradeTier('silver');
    setUpgradeReason('');
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const getUpgradeOptions = (currentTier: AwardTier): AwardTier[] => {
    switch (currentTier) {
      case 'bronze':
        return ['silver', 'gold'];
      case 'silver':
        return ['gold'];
      default:
        return [];
    }
  };

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="flex justify-between items-center mb-6">
          <div>
            <div className="flex items-center space-x-4">
              <Link
                to="/admin/merit"
                className="text-indigo-600 hover:text-indigo-800 text-sm"
              >
                ← Back to Merit Management
              </Link>
            </div>
            <h1 className="mt-2 text-2xl font-bold text-gray-900">Awards Management</h1>
            <p className="mt-1 text-sm text-gray-500">
              Create and upgrade awards for debate team members
            </p>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
          >
            + Create Award
          </button>
        </div>

        {/* Messages */}
        {error && (
          <div className="mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
            {error}
          </div>
        )}
        {successMessage && (
          <div className="mb-4 bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded">
            {successMessage}
          </div>
        )}

        {/* Awards Table */}
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold">All Awards ({total})</h2>
          </div>

          {isLoading ? (
            <div className="flex justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
            </div>
          ) : awards.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-500">No awards have been created yet.</p>
            </div>
          ) : (
            <>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Award
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Recipient
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Tier
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Awarded By
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Date
                      </th>
                      <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {awards.map((award) => {
                      const config = tierConfig[award.tier];
                      const canUpgrade = award.tier !== 'gold';
                      return (
                        <tr key={award.id} className="hover:bg-gray-50">
                          <td className="px-6 py-4">
                            <div className="font-medium text-gray-900">{award.title}</div>
                            {award.description && (
                              <div className="text-sm text-gray-500 truncate max-w-xs">
                                {award.description}
                              </div>
                            )}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <Link
                              to={`/users/${award.username}`}
                              className="text-indigo-600 hover:text-indigo-900"
                            >
                              {award.username}
                            </Link>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span
                              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium capitalize ${config.bg} ${config.text} border ${config.border}`}
                            >
                              {config.icon} {award.tier}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {award.awarded_by_username || 'System'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {formatDate(award.awarded_at)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                            <button
                              onClick={() => {
                                setEditAward(award);
                                setEditTitle(award.title);
                                setEditDescription(award.description || '');
                                setEditTier(award.tier);
                              }}
                              className="text-blue-600 hover:text-blue-900"
                            >
                              Edit
                            </button>
                            {canUpgrade && (
                              <button
                                onClick={() => {
                                  setUpgradeAward(award);
                                  setUpgradeTitle(award.title);
                                  setUpgradeTier(getUpgradeOptions(award.tier)[0]);
                                }}
                                className="text-indigo-600 hover:text-indigo-900"
                              >
                                Upgrade
                              </button>
                            )}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>

              {/* Pagination */}
              {totalPages > 1 && (
                <div className="px-6 py-4 border-t border-gray-200 flex items-center justify-between">
                  <button
                    onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                    disabled={currentPage === 1}
                    className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                  >
                    Previous
                  </button>
                  <span className="text-sm text-gray-500">
                    Page {currentPage} of {totalPages}
                  </span>
                  <button
                    onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                    disabled={currentPage === totalPages}
                    className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
                  >
                    Next
                  </button>
                </div>
              )}
            </>
          )}
        </div>

        {/* Create Award Modal */}
        {showCreateModal && (
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
              <h3 className="text-lg font-semibold mb-4">Create New Award</h3>
              <form onSubmit={handleCreateAward}>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Recipient
                    </label>
                    <select
                      value={selectedUserId}
                      onChange={(e) => setSelectedUserId(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      required
                    >
                      <option value="">Select a user...</option>
                      {users.map((user) => (
                        <option key={user.user_id} value={user.user_id}>
                          {user.username}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Award Title
                    </label>
                    <input
                      type="text"
                      value={awardTitle}
                      onChange={(e) => setAwardTitle(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      placeholder="e.g., Best Speaker Award"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Description (optional)
                    </label>
                    <textarea
                      value={awardDescription}
                      onChange={(e) => setAwardDescription(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      rows={2}
                      placeholder="Brief description of the award..."
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Tier
                    </label>
                    <div className="flex space-x-4">
                      {(['bronze', 'silver', 'gold'] as AwardTier[]).map((tier) => {
                        const config = tierConfig[tier];
                        return (
                          <label key={tier} className="flex items-center">
                            <input
                              type="radio"
                              name="tier"
                              value={tier}
                              checked={awardTier === tier}
                              onChange={(e) => setAwardTier(e.target.value as AwardTier)}
                              className="mr-2"
                            />
                            <span className={`${config.text} capitalize`}>
                              {config.icon} {tier}
                            </span>
                          </label>
                        );
                      })}
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Reason
                    </label>
                    <textarea
                      value={awardReason}
                      onChange={(e) => setAwardReason(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      rows={2}
                      placeholder="Why is this award being given?"
                      required
                    />
                  </div>
                </div>
                <div className="mt-6 flex justify-end space-x-3">
                  <button
                    type="button"
                    onClick={() => {
                      setShowCreateModal(false);
                      resetCreateForm();
                    }}
                    className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 border rounded-md"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={isCreating}
                    className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50"
                  >
                    {isCreating ? 'Creating...' : 'Create Award'}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* Upgrade Award Modal */}
        {upgradeAward && (
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
              <h3 className="text-lg font-semibold mb-4">Upgrade Award</h3>
              <p className="text-sm text-gray-600 mb-4">
                Upgrading "<strong>{upgradeAward.title}</strong>" from{' '}
                <span className={`capitalize ${tierConfig[upgradeAward.tier].text}`}>
                  {tierConfig[upgradeAward.tier].icon} {upgradeAward.tier}
                </span>
              </p>
              <form onSubmit={handleUpgradeAward}>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Award Title
                    </label>
                    <input
                      type="text"
                      value={upgradeTitle}
                      onChange={(e) => setUpgradeTitle(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      placeholder="Award title"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      New Tier
                    </label>
                    <div className="flex space-x-4">
                      {getUpgradeOptions(upgradeAward.tier).map((tier) => {
                        const config = tierConfig[tier];
                        return (
                          <label key={tier} className="flex items-center">
                            <input
                              type="radio"
                              name="upgradeTier"
                              value={tier}
                              checked={upgradeTier === tier}
                              onChange={(e) => setUpgradeTier(e.target.value as AwardTier)}
                              className="mr-2"
                            />
                            <span className={`${config.text} capitalize`}>
                              {config.icon} {tier}
                            </span>
                          </label>
                        );
                      })}
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Reason for Upgrade
                    </label>
                    <textarea
                      value={upgradeReason}
                      onChange={(e) => setUpgradeReason(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      rows={2}
                      placeholder="Why is this award being upgraded?"
                      required
                    />
                  </div>
                </div>
                <div className="mt-6 flex justify-end space-x-3">
                  <button
                    type="button"
                    onClick={() => {
                      setUpgradeAward(null);
                      resetUpgradeForm();
                    }}
                    className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 border rounded-md"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={isUpgrading}
                    className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50"
                  >
                    {isUpgrading ? 'Upgrading...' : 'Upgrade Award'}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* Edit Award Modal */}
        {editAward && (
          <div className="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
            <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
              <h3 className="text-lg font-semibold mb-4">Edit Award</h3>
              <p className="text-sm text-gray-600 mb-4">
                Editing award for <strong>{editAward.username}</strong>
              </p>
              <form onSubmit={handleEditAward}>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Award Title
                    </label>
                    <input
                      type="text"
                      value={editTitle}
                      onChange={(e) => setEditTitle(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      placeholder="Award title"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Description (optional)
                    </label>
                    <textarea
                      value={editDescription}
                      onChange={(e) => setEditDescription(e.target.value)}
                      className="w-full border rounded-md px-3 py-2"
                      rows={2}
                      placeholder="Brief description of the award..."
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Tier
                    </label>
                    <div className="flex space-x-4">
                      {(['bronze', 'silver', 'gold'] as AwardTier[]).map((tier) => {
                        const config = tierConfig[tier];
                        return (
                          <label key={tier} className="flex items-center">
                            <input
                              type="radio"
                              name="editTier"
                              value={tier}
                              checked={editTier === tier}
                              onChange={(e) => setEditTier(e.target.value as AwardTier)}
                              className="mr-2"
                            />
                            <span className={`${config.text} capitalize`}>
                              {config.icon} {tier}
                            </span>
                          </label>
                        );
                      })}
                    </div>
                  </div>
                </div>
                <div className="mt-6 flex justify-end space-x-3">
                  <button
                    type="button"
                    onClick={() => {
                      setEditAward(null);
                      resetEditForm();
                    }}
                    className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 border rounded-md"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    disabled={isEditing}
                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md disabled:opacity-50"
                  >
                    {isEditing ? 'Saving...' : 'Save Changes'}
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
