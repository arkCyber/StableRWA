'use client';

import {
  PlusIcon,
  CpuChipIcon,
  ArrowsRightLeftIcon,
  DocumentTextIcon,
} from '@heroicons/react/24/outline';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

const actions = [
  {
    name: 'Create Asset',
    description: 'Tokenize a new real-world asset',
    icon: PlusIcon,
    href: '/assets/create',
    color: 'bg-primary',
  },
  {
    name: 'AI Valuation',
    description: 'Get AI-powered asset valuation',
    icon: CpuChipIcon,
    href: '/ai/valuation',
    color: 'bg-secondary',
  },
  {
    name: 'Transfer Assets',
    description: 'Transfer tokens between wallets',
    icon: ArrowsRightLeftIcon,
    href: '/transfer',
    color: 'bg-accent',
  },
  {
    name: 'Generate Report',
    description: 'Create compliance reports',
    icon: DocumentTextIcon,
    href: '/reports',
    color: 'bg-success',
  },
];

export function QuickActions() {
  return (
    <Card className="p-5">
      <div className="mb-5">
        <h3 className="text-lg font-semibold text-foreground">Quick Actions</h3>
        <p className="text-sm text-muted-foreground mt-1">
          Common tasks and operations
        </p>
      </div>

      <div className="space-y-3">
        {actions.map((action) => (
          <Button
            key={action.name}
            variant="ghost"
            className="w-full justify-start h-auto p-3 hover:bg-muted/50"
            asChild
          >
            <a href={action.href}>
              <div className="flex items-center space-x-3">
                <div className={`flex h-10 w-10 items-center justify-center rounded-lg ${action.color}`}>
                  <action.icon className="h-5 w-5 text-white" />
                </div>
                <div className="flex-1 text-left">
                  <p className="text-sm font-medium text-foreground">{action.name}</p>
                  <p className="text-xs text-muted-foreground">{action.description}</p>
                </div>
              </div>
            </a>
          </Button>
        ))}
      </div>

      <div className="mt-5 pt-5 border-t">
        <div className="text-center">
          <p className="text-sm text-muted-foreground mb-3">Need help getting started?</p>
          <Button variant="outline" size="sm" asChild>
            <a href="/docs">
              <DocumentTextIcon className="h-4 w-4 mr-2" />
              View Documentation
            </a>
          </Button>
        </div>
      </div>
    </Card>
  );
}
