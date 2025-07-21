'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';

export function TradingPage() {
  const [selectedTab, setSelectedTab] = useState('overview');
  const [orderType, setOrderType] = useState('buy');
  const [selectedAsset, setSelectedAsset] = useState('');
  const [amount, setAmount] = useState('');
  const [price, setPrice] = useState('');

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Trading</h1>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Trade tokenized assets with real-time market data
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline">Market Data</Button>
          <Button>New Order</Button>
        </div>
      </div>

      {/* Trading metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Total Volume (24h)</p>
                <p className="text-2xl font-bold mt-1">$2.4M</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+15.3%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Active Orders</p>
                <p className="text-2xl font-bold mt-1">156</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+8.7%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Completed Trades</p>
                <p className="text-2xl font-bold mt-1">89</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+12.1%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm text-gray-600 dark:text-gray-400">Portfolio Value</p>
                <p className="text-2xl font-bold mt-1">$1.8M</p>
                <div className="flex items-center mt-2">
                  <span className="text-sm font-medium text-green-600">+5.4%</span>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Trading interface */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Order placement */}
        <Card>
          <CardHeader>
            <CardTitle>Place Order</CardTitle>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Create buy or sell orders for tokenized assets
            </p>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex space-x-2">
              <Button
                variant={orderType === 'buy' ? 'default' : 'outline'}
                onClick={() => setOrderType('buy')}
                className="flex-1"
              >
                Buy
              </Button>
              <Button
                variant={orderType === 'sell' ? 'default' : 'outline'}
                onClick={() => setOrderType('sell')}
                className="flex-1"
              >
                Sell
              </Button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="text-sm font-medium">Asset</label>
                <select
                  className="w-full mt-1 px-3 py-2 border rounded-md bg-background"
                  value={selectedAsset}
                  onChange={(e) => setSelectedAsset(e.target.value)}
                >
                  <option value="">Select an asset</option>
                  <option value="real-estate-1">Manhattan Office Complex</option>
                  <option value="gold-fund">Gold Reserve Fund</option>
                  <option value="art-collection">Vintage Art Collection</option>
                </select>
              </div>

              <div>
                <label className="text-sm font-medium">Amount</label>
                <Input
                  type="number"
                  placeholder="0.00"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  className="mt-1"
                />
              </div>

              <div>
                <label className="text-sm font-medium">Price per Token</label>
                <Input
                  type="number"
                  placeholder="0.00"
                  value={price}
                  onChange={(e) => setPrice(e.target.value)}
                  className="mt-1"
                />
              </div>

              <Button className="w-full" disabled={!selectedAsset || !amount || !price}>
                Place {orderType === 'buy' ? 'Buy' : 'Sell'} Order
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Market data */}
        <Card>
          <CardHeader>
            <CardTitle>Market Data</CardTitle>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Real-time asset prices and market information
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div>
                  <p className="font-medium">Manhattan Office</p>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Real Estate</p>
                </div>
                <div className="text-right">
                  <p className="font-medium">$2,450</p>
                  <Badge variant="default" className="text-xs">+2.3%</Badge>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div>
                  <p className="font-medium">Gold Reserve</p>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Commodities</p>
                </div>
                <div className="text-right">
                  <p className="font-medium">$1,890</p>
                  <Badge variant="default" className="text-xs">+1.8%</Badge>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div>
                  <p className="font-medium">Art Collection</p>
                  <p className="text-sm text-gray-600 dark:text-gray-400">Collectibles</p>
                </div>
                <div className="text-right">
                  <p className="font-medium">$3,200</p>
                  <Badge variant="destructive" className="text-xs">-0.5%</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Recent trades */}
        <Card>
          <CardHeader>
            <CardTitle>Recent Trades</CardTitle>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Your latest trading activity
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div className="flex items-center space-x-3">
                  <div className="p-2 rounded-full bg-green-100 dark:bg-green-900">
                    <span className="text-green-600 text-xs font-bold">BUY</span>
                  </div>
                  <div>
                    <p className="font-medium text-sm">Manhattan Office</p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      10 tokens @ $2,400
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-medium text-sm">$24,000</p>
                  <Badge variant="default" className="text-xs">Completed</Badge>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div className="flex items-center space-x-3">
                  <div className="p-2 rounded-full bg-red-100 dark:bg-red-900">
                    <span className="text-red-600 text-xs font-bold">SELL</span>
                  </div>
                  <div>
                    <p className="font-medium text-sm">Gold Reserve</p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      5 tokens @ $1,900
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-medium text-sm">$9,500</p>
                  <Badge variant="secondary" className="text-xs">Pending</Badge>
                </div>
              </div>

              <div className="flex items-center justify-between p-3 rounded-lg border">
                <div className="flex items-center space-x-3">
                  <div className="p-2 rounded-full bg-green-100 dark:bg-green-900">
                    <span className="text-green-600 text-xs font-bold">BUY</span>
                  </div>
                  <div>
                    <p className="font-medium text-sm">Art Collection</p>
                    <p className="text-xs text-gray-600 dark:text-gray-400">
                      2 tokens @ $3,150
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-medium text-sm">$6,300</p>
                  <Badge variant="default" className="text-xs">Completed</Badge>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
