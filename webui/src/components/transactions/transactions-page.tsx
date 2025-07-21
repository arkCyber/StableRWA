'use client';

import { useState } from 'react';
import { 
  MagnifyingGlassIcon, 
  ArrowUpIcon,
  ArrowDownIcon,
  BanknotesIcon,
  ClockIcon
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

// Mock data for demonstration
const mockTransactions = [
  {
    id: 'tx_001',
    type: 'Purchase',
    asset: 'Manhattan Office Building',
    user: 'John Smith',
    amount: 50000,
    tokens: 50,
    status: 'Completed',
    timestamp: '2024-01-20T14:30:00Z',
    hash: '0x1234...5678',
  },
  {
    id: 'tx_002',
    type: 'Sale',
    asset: 'Tesla Model S Collection',
    user: 'Sarah Johnson',
    amount: 25000,
    tokens: 25,
    status: 'Completed',
    timestamp: '2024-01-20T12:15:00Z',
    hash: '0x2345...6789',
  },
  {
    id: 'tx_003',
    type: 'Transfer',
    asset: 'Vintage Wine Portfolio',
    user: 'Michael Chen',
    amount: 15000,
    tokens: 15,
    status: 'Pending',
    timestamp: '2024-01-20T11:45:00Z',
    hash: '0x3456...7890',
  },
  {
    id: 'tx_004',
    type: 'Purchase',
    asset: 'Manhattan Office Building',
    user: 'Emily Davis',
    amount: 100000,
    tokens: 100,
    status: 'Failed',
    timestamp: '2024-01-20T10:20:00Z',
    hash: '0x4567...8901',
  },
];

const statusColors = {
  Completed: 'bg-success/10 text-success',
  Pending: 'bg-warning/10 text-warning',
  Failed: 'bg-destructive/10 text-destructive',
};

const typeColors = {
  Purchase: 'bg-primary/10 text-primary',
  Sale: 'bg-info/10 text-info',
  Transfer: 'bg-secondary/10 text-secondary-foreground',
};

export function TransactionsPage() {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedType, setSelectedType] = useState('All');
  const [selectedStatus, setSelectedStatus] = useState('All');

  const filteredTransactions = mockTransactions.filter(tx => {
    const matchesSearch = tx.asset.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         tx.user.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         tx.id.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesType = selectedType === 'All' || tx.type === selectedType;
    const matchesStatus = selectedStatus === 'All' || tx.status === selectedStatus;
    return matchesSearch && matchesType && matchesStatus;
  });

  const totalVolume = mockTransactions
    .filter(tx => tx.status === 'Completed')
    .reduce((sum, tx) => sum + tx.amount, 0);

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-foreground">Transactions</h1>
        <p className="mt-2 text-muted-foreground">
          Monitor all asset transactions and trading activity
        </p>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Total Volume</p>
                <p className="text-2xl font-bold">${totalVolume.toLocaleString()}</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
                <BanknotesIcon className="h-6 w-6 text-primary" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Completed</p>
                <p className="text-2xl font-bold">{mockTransactions.filter(tx => tx.status === 'Completed').length}</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-success/10 flex items-center justify-center">
                <ArrowUpIcon className="h-6 w-6 text-success" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Pending</p>
                <p className="text-2xl font-bold">{mockTransactions.filter(tx => tx.status === 'Pending').length}</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-warning/10 flex items-center justify-center">
                <ClockIcon className="h-6 w-6 text-warning" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-5">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Failed</p>
                <p className="text-2xl font-bold">{mockTransactions.filter(tx => tx.status === 'Failed').length}</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-destructive/10 flex items-center justify-center">
                <ArrowDownIcon className="h-6 w-6 text-destructive" />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Filters and search */}
      <div className="flex items-center space-x-4">
        <div className="relative flex-1 max-w-sm">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search transactions..."
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
          <option value="Purchase">Purchase</option>
          <option value="Sale">Sale</option>
          <option value="Transfer">Transfer</option>
        </select>
        <select
          className="form-input"
          value={selectedStatus}
          onChange={(e) => setSelectedStatus(e.target.value)}
        >
          <option value="All">All Status</option>
          <option value="Completed">Completed</option>
          <option value="Pending">Pending</option>
          <option value="Failed">Failed</option>
        </select>
      </div>

      {/* Transactions table */}
      <Card>
        <CardHeader>
          <CardTitle>Transaction History</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Transaction</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Type</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Asset</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">User</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Amount</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Status</th>
                  <th className="text-left py-3 px-4 font-medium text-muted-foreground">Time</th>
                </tr>
              </thead>
              <tbody>
                {filteredTransactions.map((tx) => (
                  <tr key={tx.id} className="border-b hover:bg-muted/50">
                    <td className="py-4 px-4">
                      <div>
                        <p className="font-medium">{tx.id}</p>
                        <p className="text-sm text-muted-foreground font-mono">{tx.hash}</p>
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <Badge className={typeColors[tx.type as keyof typeof typeColors]}>
                        {tx.type}
                      </Badge>
                    </td>
                    <td className="py-4 px-4">
                      <p className="font-medium">{tx.asset}</p>
                      <p className="text-sm text-muted-foreground">{tx.tokens} tokens</p>
                    </td>
                    <td className="py-4 px-4">
                      <span className="font-medium">{tx.user}</span>
                    </td>
                    <td className="py-4 px-4">
                      <span className="font-medium">${tx.amount.toLocaleString()}</span>
                    </td>
                    <td className="py-4 px-4">
                      <Badge className={statusColors[tx.status as keyof typeof statusColors]}>
                        {tx.status}
                      </Badge>
                    </td>
                    <td className="py-4 px-4">
                      <span className="text-sm">
                        {new Date(tx.timestamp).toLocaleString()}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
