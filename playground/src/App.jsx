import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import Playground from './pages/Playground'
import Workers from './pages/Workers'
import Streaming from './pages/Streaming'
import Monitoring from './pages/Monitoring'
import Examples from './pages/Examples'
import Documentation from './pages/Documentation'
import ErrorBoundary from './components/ErrorBoundary'

function App() {
  return (
    <ErrorBoundary fallbackMessage="An unexpected error occurred in the application. Please refresh the page to try again.">
      <Layout>
        <Routes>
          <Route path="/" element={<Playground />} />
          <Route path="/workers" element={<Workers />} />
          <Route path="/streaming" element={<Streaming />} />
          <Route path="/monitoring" element={<Monitoring />} />
          <Route path="/examples" element={<Examples />} />
          <Route path="/docs" element={<Documentation />} />
        </Routes>
      </Layout>
    </ErrorBoundary>
  )
}

export default App
