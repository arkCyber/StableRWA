'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

export function AnalyticsPage() {
  const [timeRange, setTimeRange] = useState('30d');

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Analytics</h1>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Performance insights and market analytics for your assets
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <select
            className="px-3 py-2 border rounded-md bg-background"
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
          >
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
            <option value="1y">Last year</option>
          </select>
          <Button variant="outline">Export Report</Button>
        </div>
      </div>

      {/* Performance metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Value Locked</p>
                <p className="text-2xl font-bold mt-1">$12.4M</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+12.5%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Active Assets</p>
                <p className="text-2xl font-bold mt-1">847</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+8.2%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Monthly Volume</p>
                <p className="text-2xl font-bold mt-1">$3.2M</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-red-600">-2.1%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Active Users</p>
                <p className="text-2xl font-bold mt-1">1,234</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+15.3%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <Card>
          <CardHeader>
            <CardTitle>Asset Performance</CardTitle>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Performance breakdown by asset category
            </p>
          </CardHeader>
          <CardContent>
            <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-lg flex items-center justify-center">
              <div className="text-center">
                <p className="text-lg font-medium text-gray-600 dark:text-gray-400 mb-2">Asset Performance Chart</p>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Chart visualization would go here
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Market Trends</CardTitle>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Historical performance and market insights
            </p>
          </CardHeader>
          <CardContent>
            <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-lg flex items-center justify-center">
              <div className="text-center">
                <p className="text-lg font-medium text-gray-600 dark:text-gray-400 mb-2">Market Trend Chart</p>
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Interactive chart showing asset performance over time would be displayed here
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Quick Insights */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Market Sentiment</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-600 mb-2">Bullish</div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Strong investor confidence with increasing asset values
              </p>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Liquidity Index</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600 mb-2">8.7/10</div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                High liquidity across most asset categories
              </p>
            </div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Risk Score</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center">
              <div className="text-3xl font-bold text-yellow-600 mb-2">Medium</div>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Balanced risk profile with diversified portfolio
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
