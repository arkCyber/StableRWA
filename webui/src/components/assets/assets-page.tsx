'use client';

import { useState } from 'react';
import { PlusIcon, MagnifyingGlassIcon, FunnelIcon } from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

// Mock data for demonstration
const mockAssets = [
  {
    id: '1',
    name: 'Manhattan Office Building',
    type: 'Real Estate',
    location: 'New York, NY',
    value: 2500000,
    tokens: 1000000,
    status: 'Active',
    created: '2024-01-15',
  },
  {
    id: '2',
    name: 'Tesla Model S Collection',
    type: 'Vehicles',
    location: 'California, CA',
    value: 450000,
    tokens: 450000,
    status: 'Pending',
    created: '2024-01-20',
  },
  {
    id: '3',
    name: 'Vintage Wine Portfolio',
    type: 'Collectibles',
    location: 'Bordeaux, France',
    value: 180000,
    tokens: 180000,
    status: 'Active',
    created: '2024-01-25',
  },
];

const statusColors = {
  Active: 'bg-success/10 text-success',
  Pending: 'bg-warning/10 text-warning',
  Inactive: 'bg-muted text-muted-foreground',
};

export function AssetsPage() {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedType, setSelectedType] = useState('All');

  const filteredAssets = mockAssets.filter(asset => {
    const matchesSearch = asset.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.location.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesType = selectedType === 'All' || asset.type === selectedType;
    return matchesSearch && matchesType;
  });

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">Assets</h1>
          <p className="mt-2 text-muted-foreground">
            Manage and monitor your tokenized real-world assets
          </p>
        </div>
        <Button className="btn-primary">
          <PlusIcon className="h-4 w-4 mr-2" />
          Create Asset
        </Button>
      </div>

      {/* Filters and search */}
      <div className="flex items-center space-x-4">
        <div className="relative flex-1 max-w-sm">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search assets..."
            className="form-input pl-10"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
        <select
          className="form-input"
          value={selectedType}
          onChange={(e) => setSelectedType(e.target.value)}
        >
          <option value="All">All Types</option>
          <option value="Real Estate">Real Estate</option>
          <option value="Vehicles">Vehicles</option>
          <option value="Collectibles">Collectibles</option>
        </select>
        <Button variant="outline">
          <FunnelIcon className="h-4 w-4 mr-2" />
          Filters
        </Button>
      </div>

      {/* Assets grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredAssets.map((asset) => (
          <Card key={asset.id} className="card-hover">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">{asset.name}</CardTitle>
                <Badge className={statusColors[asset.status as keyof typeof statusColors]}>
                  {asset.status}
                </Badge>
              </div>
              <p className="text-sm text-muted-foreground">{asset.type} • {asset.location}</p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Asset Value</span>
                  <span className="font-semibold">
                    ${asset.value.toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Token Supply</span>
                  <span className="font-semibold">
                    {asset.tokens.toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Created</span>
                  <span className="text-sm">
                    {new Date(asset.created).toLocaleDateString()}
                  </span>
                </div>
                <div className="pt-4 flex space-x-2">
                  <Button variant="outline" size="sm" className="flex-1">
                    View Details
                  </Button>
                  <Button variant="outline" size="sm" className="flex-1">
                    Manage
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Empty state */}
      {filteredAssets.length === 0 && (
        <div className="text-center py-12">
          <div className="mx-auto h-24 w-24 rounded-full bg-muted flex items-center justify-center mb-4">
            <PlusIcon className="h-12 w-12 text-muted-foreground" />
          </div>
          <h3 className="text-lg font-semibold mb-2">No assets found</h3>
          <p className="text-muted-foreground mb-4">
            {searchTerm || selectedType !== 'All' 
              ? 'Try adjusting your search or filters'
              : 'Get started by creating your first asset'
            }
          </p>
          <Button className="btn-primary">
            <PlusIcon className="h-4 w-4 mr-2" />
            Create Asset
          </Button>
        </div>
      )}
    </div>
  );
}
