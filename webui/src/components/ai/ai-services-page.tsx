'use client';

import { useState } from 'react';
import { CpuChipIcon, ChartBarIcon, ShieldCheckIcon, DocumentTextIcon } from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

const aiServices = [
  {
    id: 'valuation',
    name: 'AI Asset Valuation',
    description: 'Get accurate property valuations using machine learning models trained on market data',
    icon: ChartBarIcon,
    status: 'Available',
    accuracy: '94%',
    lastUpdated: '2024-01-20',
    features: ['Market Analysis', 'Comparable Sales', 'Trend Prediction', 'Risk Assessment'],
  },
  {
    id: 'risk-assessment',
    name: 'Risk Assessment',
    description: 'Comprehensive risk analysis for asset tokenization and investment decisions',
    icon: ShieldCheckIcon,
    status: 'Available',
    accuracy: '91%',
    lastUpdated: '2024-01-18',
    features: ['Credit Risk', 'Market Risk', 'Liquidity Risk', 'Regulatory Risk'],
  },
  {
    id: 'document-analysis',
    name: 'Document Analysis',
    description: 'Automated analysis and verification of legal documents and contracts',
    icon: DocumentTextIcon,
    status: 'Beta',
    accuracy: '88%',
    lastUpdated: '2024-01-15',
    features: ['Contract Review', 'Compliance Check', 'Data Extraction', 'Anomaly Detection'],
  },
  {
    id: 'market-intelligence',
    name: 'Market Intelligence',
    description: 'Real-time market insights and predictive analytics for investment strategies',
    icon: CpuChipIcon,
    status: 'Coming Soon',
    accuracy: 'N/A',
    lastUpdated: 'N/A',
    features: ['Price Prediction', 'Market Trends', 'Sentiment Analysis', 'News Impact'],
  },
];

const statusColors = {
  Available: 'bg-success/10 text-success',
  Beta: 'bg-warning/10 text-warning',
  'Coming Soon': 'bg-muted text-muted-foreground',
};

export function AiServicesPage() {
  const [selectedService, setSelectedService] = useState<string | null>(null);

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-foreground">AI Services</h1>
        <p className="mt-2 text-muted-foreground">
          Leverage artificial intelligence to enhance asset valuation, risk assessment, and decision making
        </p>
      </div>

      {/* Services grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {aiServices.map((service) => (
          <Card key={service.id} className="card-hover">
            <CardHeader>
              <div className="flex items-start justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-2 rounded-lg bg-primary/10">
                    <service.icon className="h-6 w-6 text-primary" />
                  </div>
                  <div>
                    <CardTitle className="text-lg">{service.name}</CardTitle>
                    <Badge className={statusColors[service.status as keyof typeof statusColors]}>
                      {service.status}
                    </Badge>
                  </div>
                </div>
              </div>
              <p className="text-sm text-muted-foreground mt-2">
                {service.description}
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {/* Metrics */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <span className="text-sm text-muted-foreground">Accuracy</span>
                    <p className="font-semibold">{service.accuracy}</p>
                  </div>
                  <div>
                    <span className="text-sm text-muted-foreground">Last Updated</span>
                    <p className="text-sm">{service.lastUpdated}</p>
                  </div>
                </div>

                {/* Features */}
                <div>
                  <span className="text-sm text-muted-foreground">Features</span>
                  <div className="flex flex-wrap gap-2 mt-2">
                    {service.features.map((feature) => (
                      <Badge key={feature} variant="secondary" className="text-xs">
                        {feature}
                      </Badge>
                    ))}
                  </div>
                </div>

                {/* Actions */}
                <div className="pt-4 flex space-x-2">
                  {service.status === 'Available' ? (
                    <>
                      <Button className="flex-1 btn-primary">
                        Use Service
                      </Button>
                      <Button variant="outline" className="flex-1">
                        Learn More
                      </Button>
                    </>
                  ) : service.status === 'Beta' ? (
                    <>
                      <Button variant="outline" className="flex-1">
                        Join Beta
                      </Button>
                      <Button variant="outline" className="flex-1">
                        Documentation
                      </Button>
                    </>
                  ) : (
                    <Button variant="outline" className="w-full" disabled>
                      Coming Soon
                    </Button>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
          <p className="text-sm text-muted-foreground">
            Common AI-powered tasks you can perform right now
          </p>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Button variant="outline" className="h-auto p-4 flex flex-col items-start space-y-2">
              <ChartBarIcon className="h-6 w-6 text-primary" />
              <div className="text-left">
                <p className="font-medium">Quick Valuation</p>
                <p className="text-xs text-muted-foreground">Get instant property estimates</p>
              </div>
            </Button>
            <Button variant="outline" className="h-auto p-4 flex flex-col items-start space-y-2">
              <ShieldCheckIcon className="h-6 w-6 text-primary" />
              <div className="text-left">
                <p className="font-medium">Risk Check</p>
                <p className="text-xs text-muted-foreground">Assess investment risks</p>
              </div>
            </Button>
            <Button variant="outline" className="h-auto p-4 flex flex-col items-start space-y-2">
              <DocumentTextIcon className="h-6 w-6 text-primary" />
              <div className="text-left">
                <p className="font-medium">Document Scan</p>
                <p className="text-xs text-muted-foreground">Analyze legal documents</p>
              </div>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Usage Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">API Calls Today</p>
                <p className="text-2xl font-bold">1,247</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-primary/10 flex items-center justify-center">
                <CpuChipIcon className="h-6 w-6 text-primary" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Valuations</p>
                <p className="text-2xl font-bold">89</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-success/10 flex items-center justify-center">
                <ChartBarIcon className="h-6 w-6 text-success" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Risk Assessments</p>
                <p className="text-2xl font-bold">34</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-warning/10 flex items-center justify-center">
                <ShieldCheckIcon className="h-6 w-6 text-warning" />
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">Documents Analyzed</p>
                <p className="text-2xl font-bold">156</p>
              </div>
              <div className="h-12 w-12 rounded-full bg-info/10 flex items-center justify-center">
                <DocumentTextIcon className="h-6 w-6 text-info" />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
