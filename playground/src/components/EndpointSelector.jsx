import { usePlaygroundStore } from '../hooks/usePlaygroundStore'
import { endpoints } from '../utils/endpoints'

export default function EndpointSelector() {
  const { selectedEndpoint, setSelectedEndpoint } = usePlaygroundStore()

  const categories = [...new Set(endpoints.map(e => e.category))]

  return (
    <div className="mb-6">
      <label className="block text-sm font-medium text-gray-700 mb-2">
        Select Endpoint
      </label>
      <select
        value={selectedEndpoint?.id || ''}
        onChange={(e) => {
          const endpoint = endpoints.find(ep => ep.id === e.target.value)
          setSelectedEndpoint(endpoint)
        }}
        className="input-field"
      >
        <option value="">Choose an endpoint...</option>
        {categories.map(category => (
          <optgroup key={category} label={category}>
            {endpoints
              .filter(e => e.category === category)
              .map(endpoint => (
                <option key={endpoint.id} value={endpoint.id}>
                  {endpoint.method} {endpoint.path} - {endpoint.name}
                </option>
              ))}
          </optgroup>
        ))}
      </select>
    </div>
  )
}
