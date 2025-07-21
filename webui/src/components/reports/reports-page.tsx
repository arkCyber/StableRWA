'use client';

import { useState } from 'react';

// Mock data for demonstration
const mockReports = [
  {
    id: 'rpt_001',
    name: 'Monthly Asset Performance Report',
    type: 'Performance',
    description: 'Comprehensive analysis of asset performance for January 2024',
    status: 'Ready',
    createdAt: '2024-01-20T10:00:00Z',
    size: '2.4 MB',
    format: 'PDF',
    downloads: 45,
  },
  {
    id: 'rpt_002',
    name: 'User Activity Summary',
    type: 'Activity',
    description: 'User engagement and transaction summary for Q4 2023',
    status: 'Ready',
    createdAt: '2024-01-19T15:30:00Z',
    size: '1.8 MB',
    format: 'PDF',
    downloads: 23,
  },
  {
    id: 'rpt_003',
    name: 'Risk Assessment Report',
    type: 'Risk',
    description: 'Portfolio risk analysis and recommendations',
    status: 'Generating',
    createdAt: '2024-01-20T14:00:00Z',
    size: '-',
    format: 'PDF',
    downloads: 0,
  },
];

const statusColors = {
  Ready: 'bg-gradient-to-r from-emerald-100/60 to-teal-100/60 dark:from-emerald-800/30 dark:to-teal-800/30 text-emerald-700 dark:text-emerald-300 border border-emerald-200/50 dark:border-emerald-700/30',
  Generating: 'bg-gradient-to-r from-amber-100/60 to-yellow-100/60 dark:from-amber-800/30 dark:to-yellow-800/30 text-amber-700 dark:text-amber-300 border border-amber-200/50 dark:border-amber-700/30',
  Failed: 'bg-gradient-to-r from-rose-100/60 to-red-100/60 dark:from-rose-800/30 dark:to-red-800/30 text-rose-700 dark:text-rose-300 border border-rose-200/50 dark:border-rose-700/30',
};

export function ReportsPage() {
  const [selectedType, setSelectedType] = useState('All');

  const filteredReports = mockReports.filter(report => {
    return selectedType === 'All' || report.type === selectedType;
  });
  return (
    <div className="space-y-8 min-h-screen bg-gradient-to-br from-stone-50 via-amber-50/30 to-orange-50/30 dark:from-stone-900/40 dark:via-amber-900/10 dark:to-orange-900/10 p-6">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-stone-800 dark:text-stone-200">Reports</h1>
          <p className="mt-2 text-stone-600 dark:text-stone-400">
            Generate and manage comprehensive reports for your assets and operations
          </p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-stone-600 to-amber-600/80 text-white rounded-lg hover:from-stone-700 hover:to-amber-700/80 shadow-md transition-all duration-200">
          + Generate Report
        </button>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-gradient-to-br from-stone-100/80 to-amber-100/60 dark:from-stone-800/30 dark:to-amber-800/20 p-6 rounded-xl shadow-md border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-stone-600 dark:text-stone-400">Total Reports</p>
              <p className="text-2xl font-bold text-stone-800 dark:text-stone-200">{mockReports.length}</p>
            </div>
            <div className="h-12 w-12 rounded-full bg-stone-200/80 dark:bg-stone-700/50 flex items-center justify-center text-stone-700 dark:text-stone-300 font-bold text-xl">
              📄
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-stone-100/80 to-emerald-100/60 dark:from-stone-800/30 dark:to-emerald-800/20 p-6 rounded-xl shadow-md border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-stone-600 dark:text-stone-400">Ready</p>
              <p className="text-2xl font-bold text-stone-800 dark:text-stone-200">{mockReports.filter(r => r.status === 'Ready').length}</p>
            </div>
            <div className="h-12 w-12 rounded-full bg-emerald-200/60 dark:bg-emerald-700/40 flex items-center justify-center text-emerald-700 dark:text-emerald-300 font-bold text-xl">
              ✅
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-stone-100/80 to-amber-100/60 dark:from-stone-800/30 dark:to-amber-800/20 p-6 rounded-xl shadow-md border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-stone-600 dark:text-stone-400">Generating</p>
              <p className="text-2xl font-bold text-stone-800 dark:text-stone-200">{mockReports.filter(r => r.status === 'Generating').length}</p>
            </div>
            <div className="h-12 w-12 rounded-full bg-amber-200/60 dark:bg-amber-700/40 flex items-center justify-center text-amber-700 dark:text-amber-300 font-bold text-xl">
              ⏳
            </div>
          </div>
        </div>
        <div className="bg-gradient-to-br from-stone-100/80 to-rose-100/60 dark:from-stone-800/30 dark:to-rose-800/20 p-6 rounded-xl shadow-md border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-stone-600 dark:text-stone-400">Downloads</p>
              <p className="text-2xl font-bold text-stone-800 dark:text-stone-200">{mockReports.reduce((sum, r) => sum + r.downloads, 0)}</p>
            </div>
            <div className="h-12 w-12 rounded-full bg-rose-200/60 dark:bg-rose-700/40 flex items-center justify-center text-rose-700 dark:text-rose-300 font-bold text-xl">
              📥
            </div>
          </div>
        </div>
      </div>

      {/* Recent Reports */}
      <div className="bg-gradient-to-br from-stone-50/80 to-amber-50/40 dark:from-stone-900/30 dark:to-amber-900/10 rounded-xl shadow-lg border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
        <div className="p-6 border-b border-stone-200/50 dark:border-stone-700/30">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-semibold text-stone-800 dark:text-stone-200">Recent Reports</h2>
            <select
              className="px-3 py-2 border border-stone-300/50 dark:border-stone-600/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-stone-400 text-sm bg-stone-50/80 dark:bg-stone-800/50 text-stone-800 dark:text-stone-200"
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
            >
              <option value="All">All Types</option>
              <option value="Performance">Performance</option>
              <option value="Activity">Activity</option>
              <option value="Risk">Risk</option>
            </select>
          </div>
        </div>
        <div className="p-6">
          <div className="space-y-4">
            {filteredReports.map((report) => (
              <div key={report.id} className="flex items-center justify-between p-4 rounded-xl border border-stone-200/50 dark:border-stone-700/30 bg-gradient-to-r from-stone-50/50 to-amber-50/30 dark:from-stone-800/20 dark:to-amber-800/10 hover:from-stone-100/60 hover:to-amber-100/40 dark:hover:from-stone-700/25 dark:hover:to-amber-700/15 transition-all duration-200 backdrop-blur-sm">
                <div className="flex items-center space-x-4">
                  <div className="h-12 w-12 rounded-xl bg-gradient-to-br from-stone-200/80 to-amber-200/60 dark:from-stone-700/50 dark:to-amber-700/40 flex items-center justify-center text-stone-700 dark:text-stone-300 font-bold text-lg shadow-sm">
                    📄
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-1">
                      <p className="font-medium text-stone-800 dark:text-stone-200">{report.name}</p>
                      <span className={`px-3 py-1 rounded-full text-xs font-medium ${statusColors[report.status as keyof typeof statusColors]}`}>
                        {report.status}
                      </span>
                    </div>
                    <p className="text-sm text-stone-600 dark:text-stone-400 mb-1">{report.description}</p>
                    <div className="flex items-center space-x-4 text-xs text-stone-500 dark:text-stone-500">
                      <span>{new Date(report.createdAt).toLocaleDateString()}</span>
                      <span>{report.size}</span>
                      <span>{report.format}</span>
                      <span>{report.downloads} downloads</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <button className="p-2 border border-stone-300/50 dark:border-stone-600/50 rounded-lg hover:bg-stone-100/60 dark:hover:bg-stone-700/40 text-stone-600 dark:text-stone-400 transition-colors duration-200">
                    👁️
                  </button>
                  <button
                    className="p-2 border border-stone-300/50 dark:border-stone-600/50 rounded-lg hover:bg-stone-100/60 dark:hover:bg-stone-700/40 disabled:opacity-50 disabled:cursor-not-allowed text-stone-600 dark:text-stone-400 transition-colors duration-200"
                    disabled={report.status !== 'Ready'}
                  >
                    📥
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Scheduled Reports */}
      <div className="bg-gradient-to-br from-stone-50/80 to-amber-50/40 dark:from-stone-900/30 dark:to-amber-900/10 rounded-xl shadow-lg border border-stone-200/50 dark:border-stone-700/30 backdrop-blur-sm">
        <div className="p-6 border-b border-stone-200/50 dark:border-stone-700/30">
          <h2 className="text-xl font-semibold text-stone-800 dark:text-stone-200">Scheduled Reports</h2>
          <p className="text-sm text-stone-600 dark:text-stone-400 mt-1">
            Automatically generated reports on a recurring schedule
          </p>
        </div>
        <div className="p-6">
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 rounded-xl border border-stone-200/50 dark:border-stone-700/30 bg-gradient-to-r from-stone-50/50 to-amber-50/30 dark:from-stone-800/20 dark:to-amber-800/10 backdrop-blur-sm">
              <div className="flex items-center space-x-4">
                <div className="h-10 w-10 rounded-xl bg-gradient-to-br from-stone-200/80 to-amber-200/60 dark:from-stone-700/50 dark:to-amber-700/40 flex items-center justify-center text-stone-700 dark:text-stone-300 font-bold text-lg shadow-sm">
                  📅
                </div>
                <div>
                  <p className="font-medium text-stone-800 dark:text-stone-200">Monthly Performance Report</p>
                  <p className="text-sm text-stone-600 dark:text-stone-400">Generated on the 1st of each month</p>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                <span className="px-3 py-1 rounded-full text-xs border border-emerald-300/50 dark:border-emerald-600/50 bg-emerald-100/60 dark:bg-emerald-800/30 text-emerald-700 dark:text-emerald-300 font-medium">Active</span>
                <button className="px-3 py-1 border border-stone-300/50 dark:border-stone-600/50 rounded-lg text-sm hover:bg-stone-100/60 dark:hover:bg-stone-700/40 text-stone-600 dark:text-stone-400 transition-colors duration-200">
                  Edit
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
