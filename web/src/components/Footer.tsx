export default function Footer() {
  return (
    <footer className="site-footer">
      <div className="container mx-auto px-6 text-center max-w-[1400px]">
        <small className="text-sm text-gray-600">© {new Date().getFullYear()} Tabrela — Built with Vite + React</small>
      </div>
    </footer>
  )
}
