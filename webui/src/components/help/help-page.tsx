'use client';

import { useState } from 'react';
import { 
  MagnifyingGlassIcon,
  QuestionMarkCircleIcon,
  BookOpenIcon,
  ChatBubbleLeftRightIcon,
  PhoneIcon,
  EnvelopeIcon,
  DocumentTextIcon,
  PlayIcon
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

// Mock data for demonstration
const faqCategories = [
  {
    name: 'Getting Started',
    icon: BookOpenIcon,
    color: 'bg-primary',
    faqs: [
      {
        question: 'How do I create my first asset token?',
        answer: 'To create your first asset token, navigate to the Assets page and click "Add Asset". Follow the guided process to upload asset details, set valuation, and configure tokenization parameters.',
      },
      {
        question: 'What types of assets can be tokenized?',
        answer: 'StableRWA supports tokenization of real estate, vehicles, collectibles, art, and other physical assets. Each asset type has specific requirements and documentation needs.',
      },
    ],
  },
  {
    name: 'Asset Management',
    icon: DocumentTextIcon,
    color: 'bg-success',
    faqs: [
      {
        question: 'How do I update asset valuations?',
        answer: 'Asset valuations can be updated through the asset detail page. Click on any asset, then use the "Update Valuation" button to submit new appraisal documents and values.',
      },
      {
        question: 'Can I transfer asset ownership?',
        answer: 'Yes, asset ownership can be transferred through the platform. The transfer process includes verification steps and may require additional documentation depending on the asset type.',
      },
    ],
  },
  {
    name: 'Trading & Transactions',
    icon: ChatBubbleLeftRightIcon,
    color: 'bg-info',
    faqs: [
      {
        question: 'How do I buy asset tokens?',
        answer: 'You can purchase asset tokens through the marketplace. Browse available assets, review details, and use the "Buy Tokens" feature to complete your purchase.',
      },
      {
        question: 'What are the transaction fees?',
        answer: 'Transaction fees vary by asset type and transaction size. Typically, fees range from 0.5% to 2% of the transaction value. Detailed fee structures are available in your account settings.',
      },
    ],
  },
];

const supportChannels = [
  {
    name: 'Live Chat',
    description: 'Get instant help from our support team',
    icon: ChatBubbleLeftRightIcon,
    color: 'bg-primary',
    availability: '24/7',
  },
  {
    name: 'Email Support',
    description: 'Send us detailed questions and feedback',
    icon: EnvelopeIcon,
    color: 'bg-success',
    availability: 'Response within 24h',
  },
  {
    name: 'Phone Support',
    description: 'Speak directly with our experts',
    icon: PhoneIcon,
    color: 'bg-info',
    availability: 'Mon-Fri 9AM-6PM EST',
  },
];

const tutorials = [
  {
    title: 'Platform Overview',
    description: 'Complete walkthrough of StableRWA features',
    duration: '15 min',
    type: 'Video',
  },
  {
    title: 'Creating Your First Asset',
    description: 'Step-by-step guide to asset tokenization',
    duration: '10 min',
    type: 'Video',
  },
  {
    title: 'Managing Your Portfolio',
    description: 'Tips for effective asset management',
    duration: '8 min',
    type: 'Article',
  },
  {
    title: 'Understanding Analytics',
    description: 'How to read and use platform analytics',
    duration: '12 min',
    type: 'Video',
  },
];

export function HelpPage() {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('All');

  const filteredFAQs = faqCategories.filter(category => {
    if (selectedCategory !== 'All' && category.name !== selectedCategory) return false;
    if (!searchTerm) return true;
    return category.faqs.some(faq => 
      faq.question.toLowerCase().includes(searchTerm.toLowerCase()) ||
      faq.answer.toLowerCase().includes(searchTerm.toLowerCase())
    );
  });

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div className="text-center">
        <h1 className="text-3xl font-bold tracking-tight text-foreground">Help Center</h1>
        <p className="mt-2 text-muted-foreground">
          Find answers, tutorials, and get support for StableRWA
        </p>
      </div>

      {/* Search */}
      <div className="max-w-2xl mx-auto">
        <div className="relative">
          <MagnifyingGlassIcon className="absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search for help articles, tutorials, or FAQs..."
            className="form-input pl-12 text-center"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {supportChannels.map((channel) => (
          <Card key={channel.name} className="card-hover cursor-pointer">
            <CardContent className="p-6 text-center">
              <div className={`h-16 w-16 rounded-full ${channel.color} flex items-center justify-center mx-auto mb-4`}>
                <channel.icon className="h-8 w-8 text-white" />
              </div>
              <h3 className="text-lg font-semibold mb-2">{channel.name}</h3>
              <p className="text-sm text-muted-foreground mb-3">{channel.description}</p>
              <Badge variant="outline">{channel.availability}</Badge>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* FAQ Categories */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle>Frequently Asked Questions</CardTitle>
                <select
                  className="form-input text-sm"
                  value={selectedCategory}
                  onChange={(e) => setSelectedCategory(e.target.value)}
                >
                  <option value="All">All Categories</option>
                  {faqCategories.map((category) => (
                    <option key={category.name} value={category.name}>
                      {category.name}
                    </option>
                  ))}
                </select>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-6">
                {filteredFAQs.map((category) => (
                  <div key={category.name}>
                    <div className="flex items-center space-x-3 mb-4">
                      <div className={`h-8 w-8 rounded-lg ${category.color} flex items-center justify-center`}>
                        <category.icon className="h-5 w-5 text-white" />
                      </div>
                      <h3 className="text-lg font-semibold">{category.name}</h3>
                    </div>
                    <div className="space-y-4 ml-11">
                      {category.faqs.map((faq, index) => (
                        <div key={index} className="border rounded-lg p-4">
                          <h4 className="font-medium mb-2 flex items-center">
                            <QuestionMarkCircleIcon className="h-5 w-5 text-primary mr-2" />
                            {faq.question}
                          </h4>
                          <p className="text-sm text-muted-foreground pl-7">{faq.answer}</p>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Tutorials & Resources */}
        <div>
          <Card>
            <CardHeader>
              <CardTitle>Tutorials & Guides</CardTitle>
              <p className="text-sm text-muted-foreground">
                Learn how to use StableRWA effectively
              </p>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {tutorials.map((tutorial, index) => (
                  <div key={index} className="flex items-start space-x-3 p-3 rounded-lg border hover:bg-muted/50 cursor-pointer">
                    <div className="h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                      {tutorial.type === 'Video' ? (
                        <PlayIcon className="h-5 w-5 text-primary" />
                      ) : (
                        <DocumentTextIcon className="h-5 w-5 text-primary" />
                      )}
                    </div>
                    <div className="flex-1">
                      <h4 className="font-medium text-sm">{tutorial.title}</h4>
                      <p className="text-xs text-muted-foreground mt-1">{tutorial.description}</p>
                      <div className="flex items-center space-x-2 mt-2">
                        <Badge variant="outline" className="text-xs">{tutorial.type}</Badge>
                        <span className="text-xs text-muted-foreground">{tutorial.duration}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Contact Info */}
          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Contact Information</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center space-x-3">
                  <EnvelopeIcon className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <p className="text-sm font-medium">Email</p>
                    <p className="text-sm text-muted-foreground">support@stablerwa.com</p>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <PhoneIcon className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <p className="text-sm font-medium">Phone</p>
                    <p className="text-sm text-muted-foreground">+1 (555) 123-4567</p>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <ChatBubbleLeftRightIcon className="h-5 w-5 text-muted-foreground" />
                  <div>
                    <p className="text-sm font-medium">Live Chat</p>
                    <p className="text-sm text-muted-foreground">Available 24/7</p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
