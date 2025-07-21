'use client';

import { useState } from 'react';
import { 
  BuildingOfficeIcon,
  TruckIcon,
  CubeIcon,
  PlusIcon,
  MagnifyingGlassIcon,
  FunnelIcon,
  EyeIcon,
  PencilIcon,
  TrashIcon
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';

// Mock data for demonstration
const assetCategories = [
  {
    name: 'Real Estate',
    count: 156,
    value: '$45.2M',
    change: '+12.5%',
    icon: BuildingOfficeIcon,
    color: 'bg-blue-500'
  },
  {
    name: 'Vehicles',
    count: 89,
    value: '$12.8M',
    change: '+8.3%',
    icon: TruckIcon,
    color: 'bg-green-500'
  },
  {
    name: 'Collectibles',
    count: 234,
    value: '$8.9M',
    change: '+15.7%',
    icon: CubeIcon,
    color: 'bg-purple-500'
  }
];

const assets = [
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
    owner: 'RWA Holdings LLC'
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
    owner: 'Fleet Management Inc'
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
    owner: 'Wine Investments Ltd'
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
    owner: 'Industrial Properties Co'
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
    owner: 'Timepiece Investments'
  }
];

export function AssetManagementPage() {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [selectedStatus, setSelectedStatus] = useState('all');

  const filteredAssets = assets.filter(asset => {
    const matchesSearch = asset.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.category.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.location.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesCategory = selectedCategory === 'all' || asset.category === selectedCategory;
    const matchesStatus = selectedStatus === 'all' || asset.status === selectedStatus;
    
    return matchesSearch && matchesCategory && matchesStatus;
  });

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'default';
      case 'pending': return 'secondary';
      case 'inactive': return 'outline';
      default: return 'default';
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'Real Estate': return BuildingOfficeIcon;
      case 'Vehicles': return TruckIcon;
      case 'Collectibles': return CubeIcon;
      default: return CubeIcon;
    }
  };

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">Asset Management</h1>
          <p className="mt-2 text-muted-foreground">
            Manage and monitor your tokenized real-world assets
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline">Import Assets</Button>
          <Button>
            <PlusIcon className="h-4 w-4 mr-2" />
            Add Asset
          </Button>
        </div>
      </div>

      {/* Asset Categories Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {assetCategories.map((category) => (
          <Card key={category.name} className="card-hover">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-2">
                    <div className={`p-2 rounded-lg ${category.color}`}>
                      <category.icon className="h-5 w-5 text-white" />
                    </div>
                    <h3 className="font-semibold">{category.name}</h3>
                  </div>
                  <div className="space-y-1">
                    <p className="text-2xl font-bold">{category.count}</p>
                    <p className="text-sm text-muted-foreground">Assets</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-lg font-semibold">{category.value}</p>
                  <p className="text-sm text-green-600">{category.change}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Search and Filters */}
      <Card>
        <CardContent className="p-6">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search assets by name, category, or location..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <div className="flex gap-2">
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="px-3 py-2 border rounded-md text-sm"
              >
                <option value="all">All Categories</option>
                <option value="Real Estate">Real Estate</option>
                <option value="Vehicles">Vehicles</option>
                <option value="Collectibles">Collectibles</option>
              </select>
              <select
                value={selectedStatus}
                onChange={(e) => setSelectedStatus(e.target.value)}
                className="px-3 py-2 border rounded-md text-sm"
              >
                <option value="all">All Status</option>
                <option value="active">Active</option>
                <option value="pending">Pending</option>
                <option value="inactive">Inactive</option>
              </select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Assets Table */}
      <Card>
        <CardHeader>
          <CardTitle>Assets ({filteredAssets.length})</CardTitle>
          <p className="text-sm text-muted-foreground">
            Manage your asset portfolio and tokenization status
          </p>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4 font-medium">Asset</th>
                  <th className="text-left py-3 px-4 font-medium">Category</th>
                  <th className="text-left py-3 px-4 font-medium">Value</th>
                  <th className="text-left py-3 px-4 font-medium">Tokenization</th>
                  <th className="text-left py-3 px-4 font-medium">Status</th>
                  <th className="text-left py-3 px-4 font-medium">Last Updated</th>
                  <th className="text-left py-3 px-4 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredAssets.map((asset) => {
                  const CategoryIcon = getCategoryIcon(asset.category);
                  return (
                    <tr key={asset.id} className="border-b hover:bg-muted/50">
                      <td className="py-4 px-4">
                        <div className="flex items-center space-x-3">
                          <div className="p-2 bg-muted rounded-lg">
                            <CategoryIcon className="h-4 w-4" />
                          </div>
                          <div>
                            <p className="font-medium">{asset.name}</p>
                            <p className="text-sm text-muted-foreground">{asset.location}</p>
                          </div>
                        </div>
                      </td>
                      <td className="py-4 px-4">
                        <Badge variant="outline">{asset.category}</Badge>
                      </td>
                      <td className="py-4 px-4">
                        <p className="font-medium">{asset.value}</p>
                      </td>
                      <td className="py-4 px-4">
                        <div className="flex items-center space-x-2">
                          {asset.tokenized ? (
                            <>
                              <Badge variant="default">Tokenized</Badge>
                              <span className="text-sm text-muted-foreground">
                                {asset.tokens} tokens
                              </span>
                            </>
                          ) : (
                            <Badge variant="secondary">Not Tokenized</Badge>
                          )}
                        </div>
                      </td>
                      <td className="py-4 px-4">
                        <Badge variant={getStatusColor(asset.status)}>
                          {asset.status}
                        </Badge>
                      </td>
                      <td className="py-4 px-4">
                        <p className="text-sm text-muted-foreground">{asset.lastUpdated}</p>
                      </td>
                      <td className="py-4 px-4">
                        <div className="flex items-center space-x-2">
                          <Button variant="ghost" size="sm">
                            <EyeIcon className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="sm">
                            <PencilIcon className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="sm">
                            <TrashIcon className="h-4 w-4" />
                          </Button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
          
          {filteredAssets.length === 0 && (
            <div className="text-center py-8">
              <CubeIcon className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <p className="text-lg font-medium text-muted-foreground mb-2">No assets found</p>
              <p className="text-sm text-muted-foreground">
                Try adjusting your search criteria or add a new asset
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card className="card-hover cursor-pointer">
          <CardContent className="p-6 text-center">
            <PlusIcon className="h-8 w-8 text-primary mx-auto mb-2" />
            <h3 className="font-medium mb-1">Add New Asset</h3>
            <p className="text-sm text-muted-foreground">Register a new real-world asset</p>
          </CardContent>
        </Card>
        
        <Card className="card-hover cursor-pointer">
          <CardContent className="p-6 text-center">
            <CubeIcon className="h-8 w-8 text-primary mx-auto mb-2" />
            <h3 className="font-medium mb-1">Tokenize Asset</h3>
            <p className="text-sm text-muted-foreground">Convert asset to digital tokens</p>
          </CardContent>
        </Card>
        
        <Card className="card-hover cursor-pointer">
          <CardContent className="p-6 text-center">
            <FunnelIcon className="h-8 w-8 text-primary mx-auto mb-2" />
            <h3 className="font-medium mb-1">Bulk Operations</h3>
            <p className="text-sm text-muted-foreground">Perform actions on multiple assets</p>
          </CardContent>
        </Card>
        
        <Card className="card-hover cursor-pointer">
          <CardContent className="p-6 text-center">
            <MagnifyingGlassIcon className="h-8 w-8 text-primary mx-auto mb-2" />
            <h3 className="font-medium mb-1">Asset Valuation</h3>
            <p className="text-sm text-muted-foreground">Get professional asset appraisal</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
