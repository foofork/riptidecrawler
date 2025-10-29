import { Component } from 'react'
import { FaExclamationTriangle, FaRedo } from 'react-icons/fa'

/**
 * Error Boundary Component
 * Catches and handles errors in child components gracefully
 */
class ErrorBoundary extends Component {
  constructor(props) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    }
  }

  static getDerivedStateFromError(error) {
    return { hasError: true }
  }

  componentDidCatch(error, errorInfo) {
    console.error('Error caught by boundary:', error, errorInfo)
    this.setState({
      error,
      errorInfo
    })
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    })
    if (this.props.onReset) {
      this.props.onReset()
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-[400px] flex items-center justify-center p-8">
          <div className="max-w-md w-full bg-red-50 border-2 border-red-200 rounded-lg p-6">
            <div className="flex items-center mb-4 text-red-600">
              <FaExclamationTriangle className="text-3xl mr-3" />
              <h2 className="text-xl font-bold">Something went wrong</h2>
            </div>

            <div className="mb-4">
              <p className="text-red-800 mb-2">
                {this.props.fallbackMessage || "We're sorry, but something unexpected happened. Please try again."}
              </p>

              {this.state.error && (
                <details className="mt-3">
                  <summary className="cursor-pointer text-sm text-red-700 hover:text-red-900 font-medium">
                    Technical Details
                  </summary>
                  <div className="mt-2 p-3 bg-red-100 rounded text-xs font-mono text-red-900 overflow-auto max-h-32">
                    <div className="mb-2">
                      <strong>Error:</strong> {this.state.error.toString()}
                    </div>
                    {this.state.errorInfo && (
                      <div>
                        <strong>Component Stack:</strong>
                        <pre className="whitespace-pre-wrap mt-1">
                          {this.state.errorInfo.componentStack}
                        </pre>
                      </div>
                    )}
                  </div>
                </details>
              )}
            </div>

            <button
              onClick={this.handleReset}
              className="btn-primary w-full flex items-center justify-center space-x-2"
            >
              <FaRedo />
              <span>Try Again</span>
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

export default ErrorBoundary
