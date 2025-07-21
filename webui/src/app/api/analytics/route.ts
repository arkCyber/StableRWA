import { NextRequest, NextResponse } from 'next/server';

// Mock analytics data
const mockAnalyticsData = {
  metrics: {
    totalAssets: {
      value: '$45.2M',
      change: '+20.1%',
      changeType: 'positive'
    },
    activeTokens: {
      value: '2,350',
      change: '+180',
      changeType: 'positive'
    },
    transactionVolume: {
      value: '$12.4M',
      change: '+19%',
      changeType: 'positive'
    },
    activeUsers: {
      value: '573',
      change: '+201',
      changeType: 'positive'
    }
  },
  chartData: {
    assetPerformance: [
      { month: 'Jan', value: 35000000 },
      { month: 'Feb', value: 37500000 },
      { month: 'Mar', value: 39200000 },
      { month: 'Apr', value: 41800000 },
      { month: 'May', value: 43100000 },
      { month: 'Jun', value: 45200000 }
    ],
    assetDistribution: [
      { category: 'Real Estate', value: 45, color: '#3B82F6' },
      { category: 'Commodities', value: 30, color: '#10B981' },
      { category: 'Art & Collectibles', value: 25, color: '#8B5CF6' }
    ]
  },
  recentEvents: [
    {
      id: 1,
      type: 'Report Generated',
      description: 'Monthly compliance report generated',
      timestamp: '2 minutes ago',
      status: 'completed'
    },
    {
      id: 2,
      type: 'Data Aggregation',
      description: 'Daily asset performance aggregation',
      timestamp: '15 minutes ago',
      status: 'completed'
    },
    {
      id: 3,
      type: 'Alert Triggered',
      description: 'High transaction volume detected',
      timestamp: '1 hour ago',
      status: 'warning'
    },
    {
      id: 4,
      type: 'Forecast Update',
      description: 'Asset price forecast model updated',
      timestamp: '2 hours ago',
      status: 'completed'
    }
  ]
};

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const timeRange = searchParams.get('timeRange') || '30d';
    const metric = searchParams.get('metric') || 'all';

    // Simulate different data based on parameters
    let responseData = { ...mockAnalyticsData };

    if (timeRange === '7d') {
      responseData.metrics.totalAssets.value = '$43.8M';
      responseData.metrics.totalAssets.change = '+15.2%';
    } else if (timeRange === '90d') {
      responseData.metrics.totalAssets.value = '$47.1M';
      responseData.metrics.totalAssets.change = '+25.8%';
    }

    if (metric === 'assets') {
      responseData = {
        ...responseData,
        chartData: {
          ...responseData.chartData,
          assetPerformance: responseData.chartData.assetPerformance.map(item => ({
            ...item,
            value: item.value * 1.1 // 10% increase for asset-specific view
          }))
        }
      };
    }

    return NextResponse.json(responseData);
  } catch (error) {
    console.error('Analytics API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch analytics data' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { query, filters, timeRange } = body;

    // Mock query execution
    const mockQueryResult = {
      queryId: `query_${Date.now()}`,
      data: [
        {
          asset: 'Manhattan Office Complex',
          value: 2500000,
          tokens: 2500,
          performance: '+12.5%'
        },
        {
          asset: 'Tesla Model S Fleet',
          value: 890000,
          tokens: 890,
          performance: '+8.3%'
        },
        {
          asset: 'Vintage Wine Collection',
          value: 340000,
          tokens: 340,
          performance: '+15.7%'
        }
      ],
      metadata: {
        executionTime: Math.floor(Math.random() * 1000) + 100,
        totalRows: 3,
        cached: Math.random() > 0.5
      }
    };

    return NextResponse.json(mockQueryResult);
  } catch (error) {
    console.error('Analytics query error:', error);
    return NextResponse.json(
      { error: 'Failed to execute analytics query' },
      { status: 500 }
    );
  }
}
