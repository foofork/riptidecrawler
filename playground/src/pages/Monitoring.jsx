import React from 'react';
import SystemHealth from '../components/monitoring/SystemHealth';
import ResourceMonitor from '../components/monitoring/ResourceMonitor';
import HealthScore from '../components/monitoring/HealthScore';

/**
 * Monitoring Page
 *
 * Comprehensive monitoring dashboard displaying:
 * - Overall health score
 * - System health status
 * - Resource utilization
 *
 * All components auto-refresh independently
 */
export default function Monitoring() {
  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Page Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">System Monitoring</h1>
          <p className="mt-2 text-sm text-gray-600">
            Real-time monitoring of system health, resources, and performance
          </p>
        </div>

        {/* Monitoring Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Health Score - Full width on mobile, 1 column on desktop */}
          <div className="lg:col-span-1">
            <HealthScore />
          </div>

          {/* System Health - Full width on mobile, 2 columns on desktop */}
          <div className="lg:col-span-2">
            <SystemHealth />
          </div>

          {/* Resource Monitor - Full width */}
          <div className="lg:col-span-3">
            <ResourceMonitor />
          </div>
        </div>

        {/* Info Panel */}
        <div className="mt-8 bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="flex items-start">
            <span className="text-2xl mr-3">ℹ️</span>
            <div>
              <h3 className="text-sm font-semibold text-blue-900 mb-1">
                Monitoring Information
              </h3>
              <ul className="text-xs text-blue-800 space-y-1">
                <li>• <strong>Health Score</strong>: Overall system health (0-100) - refreshes every 15s</li>
                <li>• <strong>System Health</strong>: Individual service status - refreshes every 30s</li>
                <li>• <strong>Resources</strong>: Browser pool, memory, queue, rate limiter - refreshes every 10s</li>
                <li>• Click 🔄 Refresh on any widget to manually update</li>
              </ul>
            </div>
          </div>
        </div>

        {/* Quick Links */}
        <div className="mt-6 grid grid-cols-1 md:grid-cols-3 gap-4">
          <a
            href="/workers"
            className="p-4 bg-white border border-gray-200 rounded-lg hover:border-blue-300 hover:shadow transition-all"
          >
            <h4 className="font-semibold text-gray-900 mb-1">Workers & Jobs</h4>
            <p className="text-xs text-gray-600">
              View active jobs, submit new tasks, track worker pool
            </p>
          </a>

          <a
            href="/streaming"
            className="p-4 bg-white border border-gray-200 rounded-lg hover:border-blue-300 hover:shadow transition-all"
          >
            <h4 className="font-semibold text-gray-900 mb-1">Live Streaming</h4>
            <p className="text-xs text-gray-600">
              Real-time crawl progress, SSE/WebSocket monitoring
            </p>
          </a>

          <a
            href="/playground"
            className="p-4 bg-white border border-gray-200 rounded-lg hover:border-blue-300 hover:shadow transition-all"
          >
            <h4 className="font-semibold text-gray-900 mb-1">API Playground</h4>
            <p className="text-xs text-gray-600">
              Test all 73 API endpoints interactively
            </p>
          </a>
        </div>
      </div>
    </div>
  );
}
