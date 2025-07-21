'use client';

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Area,
  AreaChart,
} from 'recharts';
import { Card } from '@/components/ui/card';

const data = [
  { name: 'Jan', value: 4000, volume: 2400 },
  { name: 'Feb', value: 3000, volume: 1398 },
  { name: 'Mar', value: 2000, volume: 9800 },
  { name: 'Apr', value: 2780, volume: 3908 },
  { name: 'May', value: 1890, volume: 4800 },
  { name: 'Jun', value: 2390, volume: 3800 },
  { name: 'Jul', value: 3490, volume: 4300 },
  { name: 'Aug', value: 4200, volume: 5200 },
  { name: 'Sep', value: 3800, volume: 4100 },
  { name: 'Oct', value: 4500, volume: 5800 },
  { name: 'Nov', value: 5200, volume: 6200 },
  { name: 'Dec', value: 5800, volume: 7100 },
];

export function AssetChart() {
  return (
    <Card className="p-5">
      <div className="mb-5">
        <h3 className="text-lg font-semibold text-foreground">Asset Performance</h3>
        <p className="text-sm text-muted-foreground mt-1">
          Total value locked and trading volume over time
        </p>
      </div>

      <div className="h-80">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="hsl(var(--primary))" stopOpacity={0.3} />
                <stop offset="95%" stopColor="hsl(var(--primary))" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="hsl(var(--secondary))" stopOpacity={0.3} />
                <stop offset="95%" stopColor="hsl(var(--secondary))" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
            <XAxis 
              dataKey="name" 
              stroke="hsl(var(--muted-foreground))"
              fontSize={12}
            />
            <YAxis 
              stroke="hsl(var(--muted-foreground))"
              fontSize={12}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'hsl(var(--card))',
                border: '1px solid hsl(var(--border))',
                borderRadius: '8px',
                color: 'hsl(var(--card-foreground))',
              }}
            />
            <Area
              type="monotone"
              dataKey="value"
              stroke="hsl(var(--primary))"
              fillOpacity={1}
              fill="url(#colorValue)"
              strokeWidth={2}
            />
            <Area
              type="monotone"
              dataKey="volume"
              stroke="hsl(var(--secondary))"
              fillOpacity={1}
              fill="url(#colorVolume)"
              strokeWidth={2}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      <div className="mt-4 flex items-center justify-center space-x-6 text-sm">
        <div className="flex items-center">
          <div className="h-3 w-3 rounded-full bg-primary mr-2" />
          <span className="text-muted-foreground">Total Value Locked</span>
        </div>
        <div className="flex items-center">
          <div className="h-3 w-3 rounded-full bg-secondary mr-2" />
          <span className="text-muted-foreground">Trading Volume</span>
        </div>
      </div>
    </Card>
  );
}
