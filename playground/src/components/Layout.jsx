import { Link, useLocation } from 'react-router-dom'
import { FaGithub, FaBook, FaCode, FaPlay } from 'react-icons/fa'

export default function Layout({ children }) {
  const location = useLocation()

  const isActive = (path) => location.pathname === path

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-riptide-dark text-white shadow-lg">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <div className="text-3xl">🌊</div>
              <div>
                <h1 className="text-2xl font-bold">RipTide API Playground</h1>
                <p className="text-sm text-gray-300">Interactive API Testing Environment</p>
              </div>
            </div>

            <nav className="flex items-center space-x-6">
              <Link
                to="/"
                className={`flex items-center space-x-2 hover:text-riptide-blue transition-colors ${
                  isActive('/') ? 'text-riptide-blue' : ''
                }`}
              >
                <FaPlay />
                <span>Playground</span>
              </Link>

              <Link
                to="/examples"
                className={`flex items-center space-x-2 hover:text-riptide-blue transition-colors ${
                  isActive('/examples') ? 'text-riptide-blue' : ''
                }`}
              >
                <FaCode />
                <span>Examples</span>
              </Link>

              <Link
                to="/docs"
                className={`flex items-center space-x-2 hover:text-riptide-blue transition-colors ${
                  isActive('/docs') ? 'text-riptide-blue' : ''
                }`}
              >
                <FaBook />
                <span>Docs</span>
              </Link>

              <a
                href="https://github.com/your-org/riptide-api"
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center space-x-2 hover:text-riptide-blue transition-colors"
              >
                <FaGithub />
                <span>GitHub</span>
              </a>
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1">
        {children}
      </main>

      {/* Footer */}
      <footer className="bg-gray-100 border-t border-gray-200">
        <div className="container mx-auto px-4 py-6">
          <div className="flex items-center justify-between text-sm text-gray-600">
            <p>© 2025 RipTide API. Built with React + Vite + Tailwind CSS</p>
            <div className="flex space-x-4">
              <a href="#" className="hover:text-riptide-blue">Privacy</a>
              <a href="#" className="hover:text-riptide-blue">Terms</a>
              <a href="#" className="hover:text-riptide-blue">Support</a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}
