import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { AuthProvider } from './context/AuthContext'
import Header from './components/Header'
import Footer from './components/Footer'
import InstallPrompt from './components/InstallPrompt'
import AdminRoute from './components/AdminRoute'
import ProtectedRoute from './components/ProtectedRoute'
import Home from './pages/Home'
import LoginPage from './pages/LoginPage'
import SignupPage from './pages/SignupPage'
import VerifyOtpPage from './pages/VerifyOtpPage'
import ForgotPasswordPage from './pages/ForgotPasswordPage'
import ResetPasswordPage from './pages/ResetPasswordPage'
import AboutPage from './pages/AboutPage'
import ContactPage from './pages/ContactPage'
import FAQPage from './pages/FAQPage'
import MissionPage from './pages/MissionPage'
import PrivacyPage from './pages/PrivacyPage'
import TermsPage from './pages/TermsPage'
import AccessibilityPage from './pages/AccessibilityPage'
import LoadingPage from './pages/LoadingPage'
import ErrorPage from './pages/ErrorPage'
import AdminDashboardPage from './pages/AdminDashboardPage'
import AdminMeritPage from './pages/AdminMeritPage'
import AdminAwardsPage from './pages/AdminAwardsPage'
import AdminMatchesPage from './pages/AdminMatchesPage'
import AllocationPage from './pages/AllocationPage'
import MatchDetailPage from './pages/MatchDetailPage'
import BallotPage from './pages/BallotPage'
import AdminPerformancePage from './pages/AdminPerformancePage'
import EventsPage from './pages/EventsPage'
import EventDetailPage from './pages/EventDetailPage'
import EventFormPage from './pages/EventFormPage'
import AttendanceDashboardPage from './pages/AttendanceDashboardPage'
import ProfilePage from './pages/ProfilePage'
import './App.css'

function App() {
  return (
    <BrowserRouter
      future={{
        v7_startTransition: true,
        v7_relativeSplatPath: true,
      }}
    >
      <AuthProvider>
        <div className="app-shell">
          <Header />
          <main className="site-main">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/login" element={<LoginPage />} />
              <Route path="/signup" element={<SignupPage />} />
              <Route path="/verify-otp" element={<VerifyOtpPage />} />
              <Route path="/forgot-password" element={<ForgotPasswordPage />} />
              <Route path="/reset-password" element={<ResetPasswordPage />} />
              <Route path="/about" element={<AboutPage />} />
              <Route path="/contact" element={<ContactPage />} />
              <Route path="/faq" element={<FAQPage />} />
              <Route path="/mission" element={<MissionPage />} />
              <Route path="/privacy" element={<PrivacyPage />} />
              <Route path="/terms" element={<TermsPage />} />
              <Route path="/accessibility" element={<AccessibilityPage />} />
              <Route path="/loading" element={<LoadingPage />} />
              <Route path="/error" element={<ErrorPage />} />
              {/* Admin routes */}
              <Route
                path="/admin"
                element={
                  <AdminRoute>
                    <AdminDashboardPage />
                  </AdminRoute>
                }
              />
              {/* Event routes */}
              <Route
                path="/events"
                element={
                  <ProtectedRoute>
                    <EventsPage />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/events/create"
                element={
                  <AdminRoute>
                    <EventFormPage />
                  </AdminRoute>
                }
              />
              <Route
                path="/events/:eventId"
                element={
                  <ProtectedRoute>
                    <EventDetailPage />
                  </ProtectedRoute>
                }
              />
              <Route
                path="/events/:eventId/edit"
                element={
                  <AdminRoute>
                    <EventFormPage />
                  </AdminRoute>
                }
              />
              <Route
                path="/attendance/dashboard"
                element={
                  <AdminRoute>
                    <AttendanceDashboardPage />
                  </AdminRoute>
                }
              />
              {/* Merit management (admin only) */}
              <Route
                path="/admin/merit"
                element={
                  <AdminRoute>
                    <AdminMeritPage />
                  </AdminRoute>
                }
              />
              {/* Awards management (admin only) */}
              <Route
                path="/admin/awards"
                element={
                  <AdminRoute>
                    <AdminAwardsPage />
                  </AdminRoute>
                }
              />
              {/* Match/Tabulation routes (admin only) */}
              <Route
                path="/admin/events/:eventId/matches"
                element={
                  <AdminRoute>
                    <AdminMatchesPage />
                  </AdminRoute>
                }
              />
              <Route
                path="/admin/events/:eventId/series/:seriesId/allocate"
                element={
                  <AdminRoute>
                    <AllocationPage />
                  </AdminRoute>
                }
              />
              <Route
                path="/admin/matches/:matchId"
                element={
                  <AdminRoute>
                    <MatchDetailPage />
                  </AdminRoute>
                }
              />
              <Route
                path="/admin/events/:eventId/performance"
                element={
                  <AdminRoute>
                    <AdminPerformancePage />
                  </AdminRoute>
                }
              />
              {/* Ballot page for adjudicators */}
              <Route
                path="/matches/:matchId/ballot"
                element={
                  <ProtectedRoute>
                    <BallotPage />
                  </ProtectedRoute>
                }
              />
              {/* User profile routes - publicly shareable */}
              <Route
                path="/users/:username"
                element={<ProfilePage />}
              />
              {/* fallback to home */}
              <Route path="*" element={<Home />} />
            </Routes>
          </main>
          <Footer />
          <InstallPrompt />
        </div>
      </AuthProvider>
    </BrowserRouter>
  )
}

export default App
