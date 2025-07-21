'use client';

import {
  CubeIcon,
  ArrowsRightLeftIcon,
  UserPlusIcon,
  CpuChipIcon,
} from '@heroicons/react/24/outline';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

const activities = [
  {
    id: 1,
    type: 'asset_created',
    title: 'New asset tokenized',
    description: 'Manhattan Office Building #4521 has been successfully tokenized',
    timestamp: '2 minutes ago',
    icon: CubeIcon,
    status: 'success',
  },
  {
    id: 2,
    type: 'transfer',
    title: 'Asset transfer completed',
    description: '1,000 PROP tokens transferred to 0x742d...35Cc',
    timestamp: '15 minutes ago',
    icon: ArrowsRightLeftIcon,
    status: 'success',
  },
  {
    id: 3,
    type: 'user_registered',
    title: 'New user registered',
    description: 'john.doe@example.com joined the platform',
    timestamp: '1 hour ago',
    icon: UserPlusIcon,
    status: 'info',
  },
  {
    id: 4,
    type: 'ai_valuation',
    title: 'AI valuation completed',
    description: 'Property valuation for Asset #3421 completed at $2.4M',
    timestamp: '2 hours ago',
    icon: CpuChipIcon,
    status: 'success',
  },
  {
    id: 5,
    type: 'transfer',
    title: 'Transfer pending',
    description: '500 REAL tokens transfer awaiting confirmation',
    timestamp: '3 hours ago',
    icon: ArrowsRightLeftIcon,
    status: 'warning',
  },
];

const statusColors = {
  success: 'bg-success/10 text-success',
  warning: 'bg-warning/10 text-warning',
  info: 'bg-primary/10 text-primary',
  error: 'bg-destructive/10 text-destructive',
};

export function RecentActivity() {
  return (
    <Card className="p-5">
      <div className="mb-5">
        <h3 className="text-lg font-semibold text-foreground">Recent Activity</h3>
        <p className="text-sm text-muted-foreground mt-1">
          Latest transactions and system events
        </p>
      </div>

      <div className="space-y-3">
        {activities.map((activity) => (
          <div key={activity.id} className="flex items-start space-x-3 p-3 rounded-lg hover:bg-muted/50 transition-colors">
            <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${statusColors[activity.status as keyof typeof statusColors]}`}>
              <activity.icon className="h-5 w-5" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <p className="text-sm font-medium text-foreground truncate">
                  {activity.title}
                </p>
                <Badge variant="secondary" className="ml-2 text-xs px-2 py-0.5">
                  {activity.status}
                </Badge>
              </div>
              <p className="text-sm text-muted-foreground mt-1">
                {activity.description}
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                {activity.timestamp}
              </p>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-5 text-center">
        <a
          href="/activity"
          className="text-sm text-primary hover:text-primary/80 font-medium"
        >
          View all activity →
        </a>
      </div>
    </Card>
  );
}
