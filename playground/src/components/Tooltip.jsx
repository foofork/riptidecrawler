import { useState } from 'react'
import { FaQuestionCircle } from 'react-icons/fa'

/**
 * Tooltip Component
 * Displays helpful information on hover
 */
export default function Tooltip({ content, position = 'top' }) {
  const [isVisible, setIsVisible] = useState(false)

  const positionClasses = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
  }

  const arrowClasses = {
    top: 'top-full left-1/2 -translate-x-1/2 border-t-gray-800',
    bottom: 'bottom-full left-1/2 -translate-x-1/2 border-b-gray-800',
    left: 'left-full top-1/2 -translate-y-1/2 border-l-gray-800',
    right: 'right-full top-1/2 -translate-y-1/2 border-r-gray-800',
  }

  return (
    <div className="relative inline-block">
      <button
        type="button"
        className="text-gray-400 hover:text-gray-600 transition-colors ml-1"
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        onFocus={() => setIsVisible(true)}
        onBlur={() => setIsVisible(false)}
        aria-label="Help information"
      >
        <FaQuestionCircle className="text-sm" />
      </button>

      {isVisible && (
        <>
          <div
            className={`absolute z-50 px-3 py-2 bg-gray-800 text-white text-sm rounded-lg shadow-lg whitespace-nowrap ${positionClasses[position]}`}
            role="tooltip"
          >
            {content}
            <div
              className={`absolute w-0 h-0 border-4 border-transparent ${arrowClasses[position]}`}
            />
          </div>
        </>
      )}
    </div>
  )
}
