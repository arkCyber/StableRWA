'use client';

import { StatsCards } from './stats-cards';
import { AssetChart } from './asset-chart';
import { RecentActivity } from './recent-activity';
import { QuickActions } from './quick-actions';

export function Dashboard() {
  return (
    <div className="space-y-8">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-foreground">Dashboard</h1>
        <p className="mt-2 text-base text-muted-foreground">
          Welcome to StableRWA. Monitor your assets, track performance, and manage tokenization.
        </p>
      </div>

      {/* Stats cards */}
      <StatsCards />

      {/* Main content grid */}
      <div className="grid grid-cols-1 gap-8 lg:grid-cols-3">
        {/* Asset chart - spans 2 columns on large screens */}
        <div className="lg:col-span-2">
          <AssetChart />
        </div>

        {/* Quick actions */}
        <div>
          <QuickActions />
        </div>
      </div>

      {/* Recent activity */}
      <RecentActivity />
    </div>
  );
}
