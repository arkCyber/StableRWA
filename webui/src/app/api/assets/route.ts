import { NextRequest, NextResponse } from 'next/server';

// Mock assets data
const mockAssetsData = {
  categories: [
    {
      name: 'Real Estate',
      count: 156,
      value: '$45.2M',
      change: '+12.5%',
      icon: 'building'
    },
    {
      name: 'Vehicles',
      count: 89,
      value: '$12.8M',
      change: '+8.3%',
      icon: 'truck'
    },
    {
      name: 'Collectibles',
      count: 234,
      value: '$8.9M',
      change: '+15.7%',
      icon: 'cube'
    }
  ],
  assets: [
    {
      id: 1,
      name: 'Manhattan Office Complex',
      category: 'Real Estate',
      location: 'New York, NY',
      value: '$2,500,000',
      tokenized: true,
      tokens: 2500,
      status: 'active',
      lastUpdated: '2024-01-15',
      owner: 'RWA Holdings LLC',
      description: 'Premium office space in Manhattan financial district',
      images: ['/api/placeholder/400/300'],
      documents: ['deed.pdf', 'appraisal.pdf', 'insurance.pdf'],
      performance: {
        roi: 12.5,
        appreciation: 8.3,
        rental_yield: 4.2
      }
    },
    {
      id: 2,
      name: 'Tesla Model S Fleet',
      category: 'Vehicles',
      location: 'San Francisco, CA',
      value: '$890,000',
      tokenized: true,
      tokens: 890,
      status: 'active',
      lastUpdated: '2024-01-14',
      owner: 'Fleet Management Inc',
      description: 'Fleet of 10 Tesla Model S vehicles for ride-sharing',
      images: ['/api/placeholder/400/300'],
      documents: ['registration.pdf', 'insurance.pdf', 'maintenance.pdf'],
      performance: {
        roi: 15.2,
        utilization: 85.6,
        revenue_per_mile: 1.25
      }
    },
    {
      id: 3,
      name: 'Vintage Wine Collection',
      category: 'Collectibles',
      location: 'Napa Valley, CA',
      value: '$340,000',
      tokenized: false,
      tokens: 0,
      status: 'pending',
      lastUpdated: '2024-01-13',
      owner: 'Wine Investments Ltd',
      description: 'Rare vintage wines from premium vineyards',
      images: ['/api/placeholder/400/300'],
      documents: ['authentication.pdf', 'storage.pdf', 'valuation.pdf'],
      performance: {
        roi: 22.8,
        appreciation: 18.5,
        storage_cost: 2.1
      }
    },
    {
      id: 4,
      name: 'Commercial Warehouse',
      category: 'Real Estate',
      location: 'Chicago, IL',
      value: '$1,200,000',
      tokenized: true,
      tokens: 1200,
      status: 'active',
      lastUpdated: '2024-01-12',
      owner: 'Industrial Properties Co',
      description: 'Large commercial warehouse with modern facilities',
      images: ['/api/placeholder/400/300'],
      documents: ['lease.pdf', 'inspection.pdf', 'zoning.pdf'],
      performance: {
        roi: 9.8,
        occupancy: 95.0,
        rental_yield: 6.2
      }
    },
    {
      id: 5,
      name: 'Luxury Watch Collection',
      category: 'Collectibles',
      location: 'Geneva, Switzerland',
      value: '$750,000',
      tokenized: true,
      tokens: 750,
      status: 'active',
      lastUpdated: '2024-01-11',
      owner: 'Timepiece Investments',
      description: 'Collection of rare luxury watches from top brands',
      images: ['/api/placeholder/400/300'],
      documents: ['authentication.pdf', 'insurance.pdf', 'appraisal.pdf'],
      performance: {
        roi: 18.7,
        appreciation: 16.2,
        insurance_cost: 1.5
      }
    }
  ],
  statistics: {
    totalValue: '$66.9M',
    totalAssets: 479,
    tokenizedAssets: 389,
    tokenizationRate: 81.2,
    averageROI: 15.8,
    topPerformingCategory: 'Collectibles'
  }
};

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const category = searchParams.get('category');
    const status = searchParams.get('status');
    const search = searchParams.get('search');
    const id = searchParams.get('id');

    let responseData = { ...mockAssetsData };

    // Get specific asset by ID
    if (id) {
      const asset = responseData.assets.find(a => a.id === parseInt(id));
      if (!asset) {
        return NextResponse.json(
          { error: 'Asset not found' },
          { status: 404 }
        );
      }
      return NextResponse.json({ asset });
    }

    // Filter assets based on query parameters
    let filteredAssets = responseData.assets;

    if (category && category !== 'all') {
      filteredAssets = filteredAssets.filter(
        asset => asset.category === category
      );
    }

    if (status && status !== 'all') {
      filteredAssets = filteredAssets.filter(
        asset => asset.status === status
      );
    }

    if (search) {
      const searchLower = search.toLowerCase();
      filteredAssets = filteredAssets.filter(
        asset => 
          asset.name.toLowerCase().includes(searchLower) ||
          asset.category.toLowerCase().includes(searchLower) ||
          asset.location.toLowerCase().includes(searchLower)
      );
    }

    responseData.assets = filteredAssets;

    return NextResponse.json(responseData);
  } catch (error) {
    console.error('Assets API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch assets data' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { action, data } = body;

    switch (action) {
      case 'create_asset':
        // Mock asset creation
        const newAsset = {
          id: Date.now(),
          name: data.name,
          category: data.category,
          location: data.location,
          value: data.value,
          tokenized: false,
          tokens: 0,
          status: 'pending',
          lastUpdated: new Date().toISOString().split('T')[0],
          owner: data.owner || 'Unknown',
          description: data.description || '',
          images: [],
          documents: [],
          performance: {
            roi: 0,
            appreciation: 0
          }
        };
        return NextResponse.json({ asset: newAsset });

      case 'tokenize_asset':
        // Mock asset tokenization
        const tokenizeResult = {
          assetId: data.assetId,
          tokens: data.tokens || 1000,
          tokenPrice: data.tokenPrice || '$1.00',
          status: 'tokenizing',
          estimatedCompletion: new Date(Date.now() + 300000).toISOString(), // 5 minutes
          transactionHash: `0x${Math.random().toString(16).substr(2, 64)}`
        };
        return NextResponse.json(tokenizeResult);

      case 'update_asset':
        // Mock asset update
        const updateResult = {
          assetId: data.assetId,
          updatedFields: Object.keys(data.updates),
          status: 'updated',
          lastUpdated: new Date().toISOString()
        };
        return NextResponse.json(updateResult);

      case 'delete_asset':
        // Mock asset deletion
        const deleteResult = {
          assetId: data.assetId,
          status: 'deleted',
          deletedAt: new Date().toISOString(),
          reason: data.reason || 'User requested deletion'
        };
        return NextResponse.json(deleteResult);

      case 'get_valuation':
        // Mock asset valuation
        const valuationResult = {
          assetId: data.assetId,
          currentValue: (Math.random() * 1000000 + 500000).toFixed(0),
          previousValue: data.previousValue || '1000000',
          change: ((Math.random() - 0.5) * 20).toFixed(2),
          valuationDate: new Date().toISOString(),
          methodology: 'Comparative Market Analysis',
          confidence: Math.floor(Math.random() * 20) + 80, // 80-100%
          factors: [
            'Market conditions',
            'Asset condition',
            'Location premium',
            'Recent transactions'
          ]
        };
        return NextResponse.json(valuationResult);

      case 'bulk_operation':
        // Mock bulk operations
        const bulkResult = {
          operation: data.operation,
          assetIds: data.assetIds,
          successCount: data.assetIds.length - Math.floor(Math.random() * 2),
          failureCount: Math.floor(Math.random() * 2),
          results: data.assetIds.map(id => ({
            assetId: id,
            status: Math.random() > 0.1 ? 'success' : 'failed',
            message: Math.random() > 0.1 ? 'Operation completed' : 'Operation failed'
          })),
          completedAt: new Date().toISOString()
        };
        return NextResponse.json(bulkResult);

      default:
        return NextResponse.json(
          { error: 'Unknown action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Assets action error:', error);
    return NextResponse.json(
      { error: 'Failed to execute asset action' },
      { status: 500 }
    );
  }
}
