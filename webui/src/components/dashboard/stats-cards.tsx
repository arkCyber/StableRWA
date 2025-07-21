'use client';

import {
  CubeIcon,
  CurrencyDollarIcon,
  UsersIcon,
  ChartBarIcon,
} from '@heroicons/react/24/outline';
import { Card } from '@/components/ui/card';

const stats = [
  {
    name: 'Total Assets',
    value: '2,847',
    change: '+12.5%',
    changeType: 'positive' as const,
    icon: CubeIcon,
    description: 'Assets under management',
  },
  {
    name: 'Total Value Locked',
    value: '$45.2M',
    change: '+8.2%',
    changeType: 'positive' as const,
    icon: CurrencyDollarIcon,
    description: 'USD value of tokenized assets',
  },
  {
    name: 'Active Users',
    value: '1,234',
    change: '+3.1%',
    changeType: 'positive' as const,
    icon: UsersIcon,
    description: 'Users in the last 30 days',
  },
  {
    name: 'Transaction Volume',
    value: '$12.8M',
    change: '-2.4%',
    changeType: 'negative' as const,
    icon: ChartBarIcon,
    description: 'Trading volume this month',
  },
];

export function StatsCards() {
  return (
    <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
      {stats.map((stat) => (
        <Card key={stat.name} className="p-5 card-hover">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="flex h-12 w-12 items-center justify-center rounded-lg bg-primary/10">
                <stat.icon className="h-6 w-6 text-primary" aria-hidden="true" />
              </div>
            </div>
            <div className="ml-3 flex-1">
              <div className="flex items-baseline justify-between">
                <p className="text-sm font-medium text-muted-foreground">{stat.name}</p>
                <div
                  className={`inline-flex items-baseline rounded-full px-2 py-0.5 text-sm font-medium ${
                    stat.changeType === 'positive'
                      ? 'bg-success/10 text-success'
                      : 'bg-destructive/10 text-destructive'
                  }`}
                >
                  {stat.change}
                </div>
              </div>
              <p className="text-2xl font-semibold text-foreground mt-1">{stat.value}</p>
              <p className="text-sm text-muted-foreground mt-1">{stat.description}</p>
            </div>
          </div>
        </Card>
      ))}
    </div>
  );
}
