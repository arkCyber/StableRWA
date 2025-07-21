'use client';

import { useState } from 'react';
import { 
  BookOpenIcon,
  CodeBracketIcon,
  CogIcon,
  RocketLaunchIcon,
  ShieldCheckIcon,
  ChartBarIcon,
  CubeIcon,
  BanknotesIcon,
  DocumentTextIcon,
  PlayIcon,
  ClipboardDocumentListIcon
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';

const docSections = [
  {
    id: 'getting-started',
    title: 'Getting Started',
    icon: RocketLaunchIcon,
    description: 'Quick start guide and installation instructions',
    articles: [
      { title: 'Installation Guide', description: 'Set up your development environment', time: '5 min read' },
      { title: 'First Steps', description: 'Create your first asset and start trading', time: '10 min read' },
      { title: 'Configuration', description: 'Configure your platform settings', time: '8 min read' },
      { title: 'Environment Setup', description: 'Development and production setup', time: '15 min read' }
    ]
  },
  {
    id: 'api-reference',
    title: 'API Reference',
    icon: CodeBracketIcon,
    description: 'Complete API documentation and examples',
    articles: [
      { title: 'Authentication', description: 'JWT tokens and user authentication', time: '12 min read' },
      { title: 'Assets API', description: 'Manage and tokenize real-world assets', time: '20 min read' },
      { title: 'Trading API', description: 'Place orders and execute trades', time: '18 min read' },
      { title: 'Analytics API', description: 'Access analytics and reporting data', time: '15 min read' },
      { title: 'Compliance API', description: 'KYC/AML and regulatory compliance', time: '25 min read' }
    ]
  },
  {
    id: 'asset-management',
    title: 'Asset Management',
    icon: CubeIcon,
    description: 'Learn how to manage and tokenize assets',
    articles: [
      { title: 'Asset Registration', description: 'Register new real-world assets', time: '10 min read' },
      { title: 'Tokenization Process', description: 'Convert assets into digital tokens', time: '15 min read' },
      { title: 'Asset Valuation', description: 'Automated and manual valuation methods', time: '12 min read' },
      { title: 'Lifecycle Management', description: 'Manage assets throughout their lifecycle', time: '18 min read' }
    ]
  },
  {
    id: 'trading',
    title: 'Trading Platform',
    icon: ChartBarIcon,
    description: 'Trading features and market operations',
    articles: [
      { title: 'Order Types', description: 'Market, limit, and advanced order types', time: '8 min read' },
      { title: 'Market Data', description: 'Real-time prices and market information', time: '10 min read' },
      { title: 'Portfolio Management', description: 'Track and manage your investments', time: '15 min read' },
      { title: 'Risk Management', description: 'Risk assessment and mitigation strategies', time: '20 min read' }
    ]
  },
  {
    id: 'compliance',
    title: 'Compliance & Security',
    icon: ShieldCheckIcon,
    description: 'Regulatory compliance and security features',
    articles: [
      { title: 'KYC Process', description: 'Customer verification and onboarding', time: '12 min read' },
      { title: 'AML Screening', description: 'Anti-money laundering compliance', time: '15 min read' },
      { title: 'Regulatory Reporting', description: 'Generate compliance reports', time: '18 min read' },
      { title: 'Security Best Practices', description: 'Platform security guidelines', time: '22 min read' }
    ]
  },
  {
    id: 'analytics',
    title: 'Analytics & Reporting',
    icon: DocumentTextIcon,
    description: 'Data analytics and custom reporting',
    articles: [
      { title: 'Dashboard Overview', description: 'Understanding the analytics dashboard', time: '8 min read' },
      { title: 'Custom Reports', description: 'Create and schedule custom reports', time: '15 min read' },
      { title: 'Data Export', description: 'Export data in various formats', time: '10 min read' },
      { title: 'Performance Metrics', description: 'Key performance indicators and metrics', time: '12 min read' }
    ]
  }
];

const quickLinks = [
  { title: 'API Playground', description: 'Test API endpoints interactively', icon: PlayIcon, href: '/api/playground' },
  { title: 'Code Examples', description: 'Sample code and implementations', icon: CodeBracketIcon, href: '/docs/examples' },
  { title: 'Changelog', description: 'Latest updates and changes', icon: ClipboardDocumentListIcon, href: '/docs/changelog' },
  { title: 'Support', description: 'Get help and support', icon: BookOpenIcon, href: '/help' }
];

export function DocsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSection, setSelectedSection] = useState('getting-started');

  const filteredSections = docSections.filter(section =>
    section.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    section.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    section.articles.some(article => 
      article.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      article.description.toLowerCase().includes(searchQuery.toLowerCase())
    )
  );

  const currentSection = docSections.find(section => section.id === selectedSection);

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="flex flex-col space-y-4 md:flex-row md:items-center md:justify-between md:space-y-0">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Documentation</h1>
          <p className="text-muted-foreground">
            Comprehensive guides and API reference for StableRWA platform
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Input
            placeholder="Search documentation..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-64"
          />
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Sidebar Navigation */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Documentation</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {docSections.map((section) => {
                const Icon = section.icon;
                return (
                  <button
                    key={section.id}
                    onClick={() => setSelectedSection(section.id)}
                    className={`w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-left transition-colors ${
                      selectedSection === section.id
                        ? 'bg-primary text-primary-foreground'
                        : 'hover:bg-muted'
                    }`}
                  >
                    <Icon className="h-5 w-5" />
                    <span className="text-sm font-medium">{section.title}</span>
                  </button>
                );
              })}
            </CardContent>
          </Card>

          {/* Quick Links */}
          <Card className="mt-6">
            <CardHeader>
              <CardTitle className="text-lg">Quick Links</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {quickLinks.map((link, index) => {
                const Icon = link.icon;
                return (
                  <div key={index} className="flex items-start space-x-3 p-2 rounded-lg hover:bg-muted cursor-pointer">
                    <Icon className="h-5 w-5 mt-0.5 text-muted-foreground" />
                    <div>
                      <div className="text-sm font-medium">{link.title}</div>
                      <div className="text-xs text-muted-foreground">{link.description}</div>
                    </div>
                  </div>
                );
              })}
            </CardContent>
          </Card>
        </div>

        {/* Main Content */}
        <div className="lg:col-span-3">
          {searchQuery ? (
            /* Search Results */
            <div className="space-y-6">
              <div>
                <h2 className="text-2xl font-bold mb-2">Search Results</h2>
                <p className="text-muted-foreground">
                  Found {filteredSections.length} sections matching "{searchQuery}"
                </p>
              </div>
              
              {filteredSections.map((section) => {
                const Icon = section.icon;
                return (
                  <Card key={section.id}>
                    <CardHeader>
                      <div className="flex items-center space-x-3">
                        <Icon className="h-6 w-6" />
                        <CardTitle>{section.title}</CardTitle>
                      </div>
                      <p className="text-muted-foreground">{section.description}</p>
                    </CardHeader>
                    <CardContent>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {section.articles.map((article, index) => (
                          <div key={index} className="p-4 border rounded-lg hover:bg-muted cursor-pointer">
                            <h4 className="font-medium mb-1">{article.title}</h4>
                            <p className="text-sm text-muted-foreground mb-2">{article.description}</p>
                            <Badge variant="secondary" className="text-xs">{article.time}</Badge>
                          </div>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          ) : currentSection ? (
            /* Section Content */
            <div className="space-y-6">
              <div className="flex items-center space-x-3">
                <currentSection.icon className="h-8 w-8" />
                <div>
                  <h2 className="text-2xl font-bold">{currentSection.title}</h2>
                  <p className="text-muted-foreground">{currentSection.description}</p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {currentSection.articles.map((article, index) => (
                  <Card key={index} className="hover:shadow-md transition-shadow cursor-pointer">
                    <CardContent className="p-6">
                      <div className="flex items-start justify-between mb-3">
                        <h3 className="text-lg font-semibold">{article.title}</h3>
                        <Badge variant="outline" className="text-xs">{article.time}</Badge>
                      </div>
                      <p className="text-muted-foreground mb-4">{article.description}</p>
                      <Button variant="outline" size="sm">
                        Read More
                      </Button>
                    </CardContent>
                  </Card>
                ))}
              </div>

              {/* Additional Resources */}
              <Card>
                <CardHeader>
                  <CardTitle>Additional Resources</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div className="text-center p-4 border rounded-lg">
                      <CodeBracketIcon className="h-8 w-8 mx-auto mb-2 text-primary" />
                      <h4 className="font-medium mb-1">Code Examples</h4>
                      <p className="text-sm text-muted-foreground">Sample implementations</p>
                    </div>
                    <div className="text-center p-4 border rounded-lg">
                      <PlayIcon className="h-8 w-8 mx-auto mb-2 text-primary" />
                      <h4 className="font-medium mb-1">Interactive Demos</h4>
                      <p className="text-sm text-muted-foreground">Try features live</p>
                    </div>
                    <div className="text-center p-4 border rounded-lg">
                      <BookOpenIcon className="h-8 w-8 mx-auto mb-2 text-primary" />
                      <h4 className="font-medium mb-1">Video Tutorials</h4>
                      <p className="text-sm text-muted-foreground">Step-by-step guides</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
