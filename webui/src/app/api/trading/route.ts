import { NextRequest, NextResponse } from 'next/server';

// Mock trading data
const mockTradingData = {
  metrics: {
    totalVolume24h: {
      value: '$2.4M',
      change: '+15.3%',
      changeType: 'positive'
    },
    activeOrders: {
      value: '156',
      change: '+8',
      changeType: 'positive'
    },
    completedTrades: {
      value: '89',
      change: '+12',
      changeType: 'positive'
    },
    averageTradeSize: {
      value: '$26.9K',
      change: '+5.2%',
      changeType: 'positive'
    }
  },
  recentTrades: [
    {
      id: 1,
      asset: 'Manhattan Office Complex',
      type: 'buy',
      amount: 50,
      price: '$1,000',
      total: '$50,000',
      status: 'completed',
      timestamp: '2024-01-15 14:30:25',
      trader: 'John Doe'
    },
    {
      id: 2,
      asset: 'Tesla Model S Fleet',
      type: 'sell',
      amount: 25,
      price: '$890',
      total: '$22,250',
      status: 'completed',
      timestamp: '2024-01-15 14:25:10',
      trader: 'Jane Smith'
    },
    {
      id: 3,
      asset: 'Vintage Wine Collection',
      type: 'buy',
      amount: 100,
      price: '$340',
      total: '$34,000',
      status: 'pending',
      timestamp: '2024-01-15 14:20:45',
      trader: 'Mike Johnson'
    },
    {
      id: 4,
      asset: 'Commercial Warehouse',
      type: 'sell',
      amount: 75,
      price: '$1,200',
      total: '$90,000',
      status: 'completed',
      timestamp: '2024-01-15 14:15:30',
      trader: 'Sarah Wilson'
    },
    {
      id: 5,
      asset: 'Luxury Watch Collection',
      type: 'buy',
      amount: 30,
      price: '$750',
      total: '$22,500',
      status: 'failed',
      timestamp: '2024-01-15 14:10:15',
      trader: 'David Brown'
    }
  ],
  activeOrders: [
    {
      id: 1,
      asset: 'Manhattan Office Complex',
      type: 'buy',
      amount: 100,
      price: '$950',
      total: '$95,000',
      status: 'open',
      created: '2024-01-15 13:45:00',
      expires: '2024-01-16 13:45:00'
    },
    {
      id: 2,
      asset: 'Tesla Model S Fleet',
      type: 'sell',
      amount: 50,
      price: '$920',
      total: '$46,000',
      status: 'partial',
      created: '2024-01-15 13:30:00',
      expires: '2024-01-16 13:30:00'
    },
    {
      id: 3,
      asset: 'Vintage Wine Collection',
      type: 'buy',
      amount: 200,
      price: '$320',
      total: '$64,000',
      status: 'open',
      created: '2024-01-15 13:15:00',
      expires: '2024-01-16 13:15:00'
    }
  ],
  marketSummary: [
    {
      name: 'Real Estate Tokens',
      price: '$1,250',
      change: '+2.5%',
      volume: '$450K',
      marketCap: '$125M'
    },
    {
      name: 'Vehicle Tokens',
      price: '$890',
      change: '+1.8%',
      volume: '$320K',
      marketCap: '$89M'
    },
    {
      name: 'Collectible Tokens',
      price: '$340',
      change: '+5.2%',
      volume: '$180K',
      marketCap: '$34M'
    }
  ],
  orderBook: {
    sellOrders: [
      { price: '$1,250', amount: '50', total: '$62,500' },
      { price: '$1,240', amount: '75', total: '$93,000' },
      { price: '$1,230', amount: '100', total: '$123,000' }
    ],
    buyOrders: [
      { price: '$1,200', amount: '80', total: '$96,000' },
      { price: '$1,190', amount: '120', total: '$142,800' },
      { price: '$1,180', amount: '150', total: '$177,000' }
    ]
  }
};

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const section = searchParams.get('section') || 'overview';
    const asset = searchParams.get('asset');

    let responseData = { ...mockTradingData };

    // Filter by asset if specified
    if (asset) {
      responseData.recentTrades = responseData.recentTrades.filter(
        trade => trade.asset.toLowerCase().includes(asset.toLowerCase())
      );
      responseData.activeOrders = responseData.activeOrders.filter(
        order => order.asset.toLowerCase().includes(asset.toLowerCase())
      );
    }

    // Return specific section data
    switch (section) {
      case 'orders':
        return NextResponse.json({ 
          activeOrders: responseData.activeOrders,
          orderBook: responseData.orderBook
        });
      case 'history':
        return NextResponse.json({ trades: responseData.recentTrades });
      case 'market':
        return NextResponse.json({ 
          marketSummary: responseData.marketSummary,
          orderBook: responseData.orderBook
        });
      default:
        return NextResponse.json(responseData);
    }
  } catch (error) {
    console.error('Trading API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch trading data' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { action, data } = body;

    switch (action) {
      case 'place_order':
        // Mock order placement
        const orderResult = {
          orderId: `order_${Date.now()}`,
          asset: data.asset,
          type: data.type,
          amount: data.amount,
          price: data.price,
          total: (parseFloat(data.amount) * parseFloat(data.price)).toFixed(2),
          status: 'pending',
          created: new Date().toISOString(),
          expires: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString() // 24 hours
        };

        // Simulate order processing
        setTimeout(() => {
          const success = Math.random() > 0.1; // 90% success rate
          orderResult.status = success ? 'open' : 'failed';
        }, 2000);

        return NextResponse.json(orderResult);

      case 'cancel_order':
        // Mock order cancellation
        const cancelResult = {
          orderId: data.orderId,
          status: 'cancelled',
          cancelledAt: new Date().toISOString(),
          reason: 'User requested cancellation'
        };
        return NextResponse.json(cancelResult);

      case 'get_order_status':
        // Mock order status check
        const statusResult = {
          orderId: data.orderId,
          status: ['open', 'partial', 'completed', 'cancelled'][Math.floor(Math.random() * 4)],
          filled: Math.floor(Math.random() * 100),
          remaining: Math.floor(Math.random() * 100),
          lastUpdated: new Date().toISOString()
        };
        return NextResponse.json(statusResult);

      case 'get_market_data':
        // Mock real-time market data
        const marketData = {
          asset: data.asset || 'Manhattan Office Complex',
          currentPrice: (1000 + Math.random() * 100 - 50).toFixed(2),
          change24h: ((Math.random() - 0.5) * 10).toFixed(2),
          volume24h: (Math.random() * 1000000).toFixed(0),
          high24h: (1050 + Math.random() * 50).toFixed(2),
          low24h: (950 + Math.random() * 50).toFixed(2),
          timestamp: new Date().toISOString()
        };
        return NextResponse.json(marketData);

      case 'execute_trade':
        // Mock trade execution
        const tradeResult = {
          tradeId: `trade_${Date.now()}`,
          orderId: data.orderId,
          asset: data.asset,
          type: data.type,
          amount: data.amount,
          executedPrice: data.price,
          executedAmount: data.amount,
          total: (parseFloat(data.amount) * parseFloat(data.price)).toFixed(2),
          fees: (parseFloat(data.amount) * parseFloat(data.price) * 0.001).toFixed(2), // 0.1% fee
          status: 'completed',
          executedAt: new Date().toISOString()
        };
        return NextResponse.json(tradeResult);

      default:
        return NextResponse.json(
          { error: 'Unknown action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Trading action error:', error);
    return NextResponse.json(
      { error: 'Failed to execute trading action' },
      { status: 500 }
    );
  }
}
