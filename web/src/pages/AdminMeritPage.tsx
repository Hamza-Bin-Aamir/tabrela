import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { AdminMeritService } from '../services/merit';
import type { UserMeritInfo, MeritHistoryEntry } from '../services/types';

export default function AdminMeritPage() {
  const [users, setUsers] = useState<UserMeritInfo[]>([]);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  
  // Modal state for updating merit
  const [selectedUser, setSelectedUser] = useState<UserMeritInfo | null>(null);
  const [changeAmount, setChangeAmount] = useState<string>('');
  const [reason, setReason] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  
  // History modal state
  const [historyUser, setHistoryUser] = useState<UserMeritInfo | null>(null);
  const [history, setHistory] = useState<MeritHistoryEntry[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyPage, setHistoryPage] = useState(1);
  const [historyTotalPages, setHistoryTotalPages] = useState(1);

  const perPage = 20;

  const loadUsers = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await AdminMeritService.listAllMerits(currentPage, perPage);
      setUsers(response.users);
      setTotalPages(response.total_pages);
      setTotal(response.total);
    } catch (err) {
      setError('Failed to load users. Please try again.');
      console.error('Failed to load users:', err);
    } finally {
      setIsLoading(false);
    }
  }, [currentPage]);

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  const loadHistory = useCallback(async () => {
    if (!historyUser) return;
    
    setHistoryLoading(true);
    try {
      const response = await AdminMeritService.getUserMeritHistory(
        historyUser.user_id,
        historyPage,
        10
      );
      setHistory(response.history);
      setHistoryTotalPages(response.total_pages);
    } catch (err) {
      console.error('Failed to load history:', err);
    } finally {
      setHistoryLoading(false);
    }
  }, [historyUser, historyPage]);

  useEffect(() => {
    if (historyUser) {
      loadHistory();
    }
  }, [historyUser, loadHistory]);

  const handleUpdateMerit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedUser || !changeAmount || !reason.trim()) return;

    setIsSubmitting(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const result = await AdminMeritService.updateMerit({
        user_id: selectedUser.user_id,
        change_amount: parseInt(changeAmount, 10),
        reason: reason.trim(),
      });
      
      setSuccessMessage(
        `Merit updated for ${result.username}: ${result.previous_merit} → ${result.new_merit}`
      );
      setSelectedUser(null);
      setChangeAmount('');
      setReason('');
      await loadUsers();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to update merit');
    } finally {
      setIsSubmitting(false);
    }
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

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="mb-8 flex justify-between items-start">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Merit Management</h1>
            <p className="mt-2 text-gray-600">
              View and manage merit points for all users
            </p>
          </div>
          <Link
            to="/admin/awards"
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
          >
            🏆 Manage Awards
          </Link>
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

        {/* Stats */}
        <div className="mb-6 bg-white shadow rounded-lg p-6">
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <p className="text-sm text-gray-500">Total Users</p>
              <p className="text-2xl font-bold text-gray-900">{total}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Average Merit</p>
              <p className="text-2xl font-bold text-indigo-600">
                {users.length > 0
                  ? Math.round(users.reduce((acc, u) => acc + u.merit_points, 0) / users.length)
                  : 0}
              </p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Top Merit</p>
              <p className="text-2xl font-bold text-green-600">
                {users.length > 0 ? Math.max(...users.map((u) => u.merit_points)) : 0}
              </p>
            </div>
          </div>
        </div>

        {/* Users Table */}
        <div className="bg-white shadow rounded-lg overflow-hidden">
          {isLoading ? (
            <div className="p-8 flex justify-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
            </div>
          ) : (
            <>
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      User
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Merit Points
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {users.map((user) => (
                    <tr key={user.user_id} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap">
                        <Link
                          to={`/users/${user.username}`}
                          className="text-indigo-600 hover:text-indigo-900 font-medium"
                        >
                          {user.username}
                        </Link>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span
                          className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                            user.merit_points > 0
                              ? 'bg-green-100 text-green-800'
                              : user.merit_points < 0
                              ? 'bg-red-100 text-red-800'
                              : 'bg-gray-100 text-gray-800'
                          }`}
                        >
                          {user.merit_points}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                        <button
                          onClick={() => setSelectedUser(user)}
                          className="text-indigo-600 hover:text-indigo-900"
                        >
                          Update
                        </button>
                        <button
                          onClick={() => {
                            setHistoryUser(user);
                            setHistoryPage(1);
                          }}
                          className="text-gray-600 hover:text-gray-900"
                        >
                          History
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>

              {/* Pagination */}
              <div className="bg-gray-50 px-6 py-3 flex items-center justify-between border-t border-gray-200">
                <div className="text-sm text-gray-500">
                  Showing {(currentPage - 1) * perPage + 1} to{' '}
                  {Math.min(currentPage * perPage, total)} of {total} users
                </div>
                <div className="flex space-x-2">
                  <button
                    onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                    disabled={currentPage === 1}
                    className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100"
                  >
                    Previous
                  </button>
                  <button
                    onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
                    disabled={currentPage === totalPages}
                    className="px-3 py-1 text-sm border rounded disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100"
                  >
                    Next
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Update Merit Modal */}
      {selectedUser && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-md w-full p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Update Merit for {selectedUser.username}
            </h3>
            <p className="text-sm text-gray-500 mb-4">
              Current merit: <span className="font-medium">{selectedUser.merit_points}</span>
            </p>
            
            <form onSubmit={handleUpdateMerit}>
              <div className="mb-4">
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Change Amount
                </label>
                <input
                  type="number"
                  value={changeAmount}
                  onChange={(e) => setChangeAmount(e.target.value)}
                  placeholder="e.g., 10 or -5"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  required
                />
                <p className="mt-1 text-xs text-gray-500">
                  Use positive numbers to add merit, negative to remove
                </p>
              </div>
              
              <div className="mb-6">
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Reason (required)
                </label>
                <textarea
                  value={reason}
                  onChange={(e) => setReason(e.target.value)}
                  placeholder="Explain why merit is being changed..."
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  required
                  minLength={3}
                  maxLength={500}
                />
              </div>
              
              <div className="flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => {
                    setSelectedUser(null);
                    setChangeAmount('');
                    setReason('');
                  }}
                  className="px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting || !changeAmount || !reason.trim()}
                  className="px-4 py-2 text-sm text-white bg-indigo-600 hover:bg-indigo-700 rounded-md disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isSubmitting ? 'Updating...' : 'Update Merit'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* History Modal */}
      {historyUser && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-2xl w-full p-6 max-h-[80vh] overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h3 className="text-lg font-semibold text-gray-900">
                Merit History for {historyUser.username}
              </h3>
              <button
                onClick={() => {
                  setHistoryUser(null);
                  setHistory([]);
                }}
                className="text-gray-400 hover:text-gray-600"
              >
                <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            {historyLoading ? (
              <div className="flex justify-center py-8">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
              </div>
            ) : history.length === 0 ? (
              <p className="text-center text-gray-500 py-8">No merit changes recorded.</p>
            ) : (
              <>
                <table className="min-w-full divide-y divide-gray-200">
                  <thead>
                    <tr>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                        Date
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                        Change
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                        Total
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                        By
                      </th>
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">
                        Reason
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-200">
                    {history.map((entry) => (
                      <tr key={entry.id}>
                        <td className="px-4 py-2 text-sm text-gray-500 whitespace-nowrap">
                          {formatDateTime(entry.created_at)}
                        </td>
                        <td className="px-4 py-2 text-sm">
                          <span
                            className={`inline-flex px-2 py-0.5 rounded text-sm font-medium ${
                              entry.change_amount > 0
                                ? 'bg-green-100 text-green-800'
                                : 'bg-red-100 text-red-800'
                            }`}
                          >
                            {entry.change_amount > 0 ? '+' : ''}
                            {entry.change_amount}
                          </span>
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-900">
                          {entry.new_total}
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-500 whitespace-nowrap">
                          {entry.admin_username || 'System'}
                        </td>
                        <td className="px-4 py-2 text-sm text-gray-500">
                          {entry.reason}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                
                {historyTotalPages > 1 && (
                  <div className="mt-4 flex items-center justify-between">
                    <button
                      onClick={() => setHistoryPage((p) => Math.max(1, p - 1))}
                      disabled={historyPage === 1}
                      className="px-3 py-1 text-sm border rounded disabled:opacity-50"
                    >
                      Previous
                    </button>
                    <span className="text-sm text-gray-500">
                      Page {historyPage} of {historyTotalPages}
                    </span>
                    <button
                      onClick={() => setHistoryPage((p) => Math.min(historyTotalPages, p + 1))}
                      disabled={historyPage === historyTotalPages}
                      className="px-3 py-1 text-sm border rounded disabled:opacity-50"
                    >
                      Next
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
