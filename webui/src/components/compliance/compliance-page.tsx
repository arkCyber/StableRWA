'use client';

import { useState } from 'react';
import { 
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  DocumentTextIcon,
  UserGroupIcon,
  ClockIcon,
  CheckCircleIcon
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';

// Mock data for demonstration
const complianceMetrics = [
  {
    name: 'KYC Completion Rate',
    value: '98.5%',
    target: '95%',
    status: 'excellent',
    icon: UserGroupIcon,
  },
  {
    name: 'AML Screening',
    value: '100%',
    target: '100%',
    status: 'excellent',
    icon: ShieldCheckIcon,
  },
  {
    name: 'Regulatory Filings',
    value: '92%',
    target: '95%',
    status: 'warning',
    icon: DocumentTextIcon,
  },
  {
    name: 'Audit Compliance',
    value: '96%',
    target: '90%',
    status: 'excellent',
    icon: CheckCircleIcon,
  },
];

const recentAlerts = [
  {
    id: 1,
    type: 'KYC Expiry',
    message: '15 customer KYC documents expiring within 30 days',
    severity: 'warning',
    timestamp: '2 hours ago',
    status: 'pending'
  },
  {
    id: 2,
    type: 'AML Alert',
    message: 'Suspicious transaction pattern detected for user ID: 12345',
    severity: 'high',
    timestamp: '4 hours ago',
    status: 'investigating'
  },
  {
    id: 3,
    type: 'Regulatory Update',
    message: 'New SEC guidelines for tokenized assets published',
    severity: 'info',
    timestamp: '1 day ago',
    status: 'reviewed'
  },
  {
    id: 4,
    type: 'Audit Requirement',
    message: 'Quarterly audit documentation due in 7 days',
    severity: 'warning',
    timestamp: '2 days ago',
    status: 'in_progress'
  }
];

const jurisdictions = [
  { name: 'United States', status: 'compliant', lastReview: '2024-01-15' },
  { name: 'European Union', status: 'compliant', lastReview: '2024-01-10' },
  { name: 'United Kingdom', status: 'pending_review', lastReview: '2023-12-20' },
  { name: 'Singapore', status: 'compliant', lastReview: '2024-01-05' },
  { name: 'Japan', status: 'non_compliant', lastReview: '2023-11-30' },
];

export function CompliancePage() {
  const [selectedTab, setSelectedTab] = useState('overview');

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'excellent': return 'text-green-600';
      case 'warning': return 'text-yellow-600';
      case 'critical': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'high': return 'destructive';
      case 'warning': return 'secondary';
      case 'info': return 'outline';
      default: return 'default';
    }
  };

  const getJurisdictionStatus = (status: string) => {
    switch (status) {
      case 'compliant': return { color: 'bg-green-100 text-green-800', label: 'Compliant' };
      case 'pending_review': return { color: 'bg-yellow-100 text-yellow-800', label: 'Pending Review' };
      case 'non_compliant': return { color: 'bg-red-100 text-red-800', label: 'Non-Compliant' };
      default: return { color: 'bg-gray-100 text-gray-800', label: 'Unknown' };
    }
  };

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-foreground">Compliance Management</h1>
          <p className="mt-2 text-muted-foreground">
            Monitor regulatory compliance, KYC/AML status, and audit requirements
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline">Generate Report</Button>
          <Button>Run Compliance Check</Button>
        </div>
      </div>

      {/* Compliance Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {complianceMetrics.map((metric) => {
          const IconComponent = metric.icon;
          return (
            <Card key={metric.name} className="card-hover">
              <CardContent className="p-5">
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <p className="text-sm text-muted-foreground">{metric.name}</p>
                    <p className="text-2xl font-bold mt-1">{metric.value}</p>
                    <div className="flex items-center mt-2">
                      <span className={`text-sm font-medium ${getStatusColor(metric.status)}`}>
                        Target: {metric.target}
                      </span>
                    </div>
                  </div>
                  <div className="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
                    <IconComponent className="h-6 w-6 text-primary" />
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Navigation Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview', name: 'Overview' },
            { id: 'alerts', name: 'Alerts & Notifications' },
            { id: 'jurisdictions', name: 'Jurisdictions' },
            { id: 'reports', name: 'Reports' }
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setSelectedTab(tab.id)}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                selectedTab === tab.id
                  ? 'border-primary text-primary'
                  : 'border-transparent text-muted-foreground hover:text-foreground hover:border-gray-300'
              }`}
            >
              {tab.name}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      {selectedTab === 'overview' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Compliance Status */}
          <Card>
            <CardHeader>
              <CardTitle>Overall Compliance Status</CardTitle>
              <p className="text-sm text-muted-foreground">
                Current compliance score across all categories
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">KYC Compliance</span>
                  <span className="text-sm text-muted-foreground">98.5%</span>
                </div>
                <Progress value={98.5} className="h-2" />
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">AML Screening</span>
                  <span className="text-sm text-muted-foreground">100%</span>
                </div>
                <Progress value={100} className="h-2" />
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Regulatory Filings</span>
                  <span className="text-sm text-muted-foreground">92%</span>
                </div>
                <Progress value={92} className="h-2" />
                
                <div className="flex items-center justify-between">
                  <span className="text-sm font-medium">Audit Readiness</span>
                  <span className="text-sm text-muted-foreground">96%</span>
                </div>
                <Progress value={96} className="h-2" />
              </div>
              
              <div className="mt-6 p-4 bg-green-50 rounded-lg">
                <div className="flex items-center">
                  <CheckCircleIcon className="h-5 w-5 text-green-600 mr-2" />
                  <span className="text-sm font-medium text-green-800">
                    Overall Status: Compliant
                  </span>
                </div>
                <p className="text-sm text-green-700 mt-1">
                  All critical compliance requirements are met
                </p>
              </div>
            </CardContent>
          </Card>

          {/* Recent Activity */}
          <Card>
            <CardHeader>
              <CardTitle>Recent Compliance Activity</CardTitle>
              <p className="text-sm text-muted-foreground">
                Latest compliance checks and updates
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {recentAlerts.slice(0, 4).map((alert) => (
                  <div key={alert.id} className="flex items-start space-x-3 p-3 rounded-lg border">
                    <div className="flex-shrink-0">
                      {alert.severity === 'high' ? (
                        <ExclamationTriangleIcon className="h-5 w-5 text-red-500" />
                      ) : alert.severity === 'warning' ? (
                        <ClockIcon className="h-5 w-5 text-yellow-500" />
                      ) : (
                        <DocumentTextIcon className="h-5 w-5 text-blue-500" />
                      )}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-foreground">{alert.type}</p>
                      <p className="text-sm text-muted-foreground">{alert.message}</p>
                      <div className="flex items-center mt-2 space-x-2">
                        <Badge variant={getSeverityColor(alert.severity)} className="text-xs">
                          {alert.status.replace('_', ' ')}
                        </Badge>
                        <span className="text-xs text-muted-foreground">{alert.timestamp}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {selectedTab === 'alerts' && (
        <Card>
          <CardHeader>
            <CardTitle>Compliance Alerts & Notifications</CardTitle>
            <p className="text-sm text-muted-foreground">
              All compliance-related alerts and notifications
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {recentAlerts.map((alert) => (
                <div key={alert.id} className="flex items-center justify-between p-4 border rounded-lg">
                  <div className="flex items-start space-x-3">
                    <div className="flex-shrink-0">
                      {alert.severity === 'high' ? (
                        <ExclamationTriangleIcon className="h-5 w-5 text-red-500" />
                      ) : alert.severity === 'warning' ? (
                        <ClockIcon className="h-5 w-5 text-yellow-500" />
                      ) : (
                        <DocumentTextIcon className="h-5 w-5 text-blue-500" />
                      )}
                    </div>
                    <div>
                      <p className="text-sm font-medium text-foreground">{alert.type}</p>
                      <p className="text-sm text-muted-foreground">{alert.message}</p>
                      <span className="text-xs text-muted-foreground">{alert.timestamp}</span>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Badge variant={getSeverityColor(alert.severity)}>
                      {alert.severity}
                    </Badge>
                    <Button variant="outline" size="sm">
                      Review
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {selectedTab === 'jurisdictions' && (
        <Card>
          <CardHeader>
            <CardTitle>Jurisdiction Compliance Status</CardTitle>
            <p className="text-sm text-muted-foreground">
              Compliance status across different regulatory jurisdictions
            </p>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {jurisdictions.map((jurisdiction) => {
                const statusInfo = getJurisdictionStatus(jurisdiction.status);
                return (
                  <div key={jurisdiction.name} className="flex items-center justify-between p-4 border rounded-lg">
                    <div>
                      <p className="font-medium">{jurisdiction.name}</p>
                      <p className="text-sm text-muted-foreground">
                        Last reviewed: {jurisdiction.lastReview}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${statusInfo.color}`}>
                        {statusInfo.label}
                      </span>
                      <Button variant="outline" size="sm">
                        Review
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {selectedTab === 'reports' && (
        <Card>
          <CardHeader>
            <CardTitle>Compliance Reports</CardTitle>
            <p className="text-sm text-muted-foreground">
              Generate and manage compliance reports
            </p>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {[
                { name: 'KYC Summary Report', description: 'Customer verification status summary', frequency: 'Monthly' },
                { name: 'AML Screening Report', description: 'Anti-money laundering screening results', frequency: 'Weekly' },
                { name: 'Regulatory Filing Report', description: 'Status of regulatory submissions', frequency: 'Quarterly' },
                { name: 'Audit Trail Report', description: 'Complete audit trail documentation', frequency: 'On-demand' },
                { name: 'Risk Assessment Report', description: 'Comprehensive risk analysis', frequency: 'Monthly' },
                { name: 'Jurisdiction Compliance Report', description: 'Multi-jurisdiction compliance status', frequency: 'Quarterly' }
              ].map((report) => (
                <div key={report.name} className="p-4 border rounded-lg">
                  <h3 className="font-medium mb-2">{report.name}</h3>
                  <p className="text-sm text-muted-foreground mb-3">{report.description}</p>
                  <div className="flex items-center justify-between">
                    <Badge variant="outline" className="text-xs">
                      {report.frequency}
                    </Badge>
                    <Button variant="outline" size="sm">
                      Generate
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
