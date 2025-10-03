import { Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import Playground from './pages/Playground'
import Examples from './pages/Examples'
import Documentation from './pages/Documentation'

function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Playground />} />
        <Route path="/examples" element={<Examples />} />
        <Route path="/docs" element={<Documentation />} />
      </Routes>
    </Layout>
  )
}

export default App
