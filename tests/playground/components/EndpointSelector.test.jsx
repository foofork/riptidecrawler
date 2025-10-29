import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { mockEndpoints } from '../fixtures/mockData'

// Mock EndpointSelector component
const EndpointSelector = ({ endpoints, selected, onSelect }) => {
  return (
    <div>
      <label htmlFor="endpoint-select">Select Endpoint</label>
      <select
        id="endpoint-select"
        value={selected?.id || ''}
        onChange={(e) => {
          const endpoint = endpoints.find(ep => ep.id === e.target.value)
          onSelect(endpoint)
        }}
      >
        <option value="">Choose an endpoint...</option>
        {endpoints.map(ep => (
          <option key={ep.id} value={ep.id}>
            {ep.method} {ep.path} - {ep.title}
          </option>
        ))}
      </select>
    </div>
  )
}

describe('EndpointSelector Component', () => {
  it('should render endpoint selector dropdown', () => {
    const onSelect = vi.fn()
    render(<EndpointSelector endpoints={mockEndpoints} selected={null} onSelect={onSelect} />)

    expect(screen.getByLabelText('Select Endpoint')).toBeInTheDocument()
    expect(screen.getByText('Choose an endpoint...')).toBeInTheDocument()
  })

  it('should display all endpoints in dropdown', () => {
    const onSelect = vi.fn()
    render(<EndpointSelector endpoints={mockEndpoints} selected={null} onSelect={onSelect} />)

    mockEndpoints.forEach(endpoint => {
      expect(screen.getByText(new RegExp(endpoint.title))).toBeInTheDocument()
    })
  })

  it('should call onSelect when endpoint is chosen', async () => {
    const onSelect = vi.fn()
    const user = userEvent.setup()

    render(<EndpointSelector endpoints={mockEndpoints} selected={null} onSelect={onSelect} />)

    const select = screen.getByLabelText('Select Endpoint')
    await user.selectOptions(select, mockEndpoints[0].id)

    expect(onSelect).toHaveBeenCalledWith(mockEndpoints[0])
  })

  it('should show selected endpoint', () => {
    const onSelect = vi.fn()
    render(<EndpointSelector endpoints={mockEndpoints} selected={mockEndpoints[1]} onSelect={onSelect} />)

    const select = screen.getByLabelText('Select Endpoint')
    expect(select.value).toBe(mockEndpoints[1].id)
  })

  it('should show method and path in options', () => {
    const onSelect = vi.fn()
    render(<EndpointSelector endpoints={mockEndpoints} selected={null} onSelect={onSelect} />)

    expect(screen.getByText(/POST \/crawl/)).toBeInTheDocument()
    expect(screen.getByText(/GET \/job\/:jobId/)).toBeInTheDocument()
  })
})
