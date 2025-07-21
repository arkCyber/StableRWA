'use client';

import { useState } from 'react';
import { 
  UserIcon, 
  ShieldCheckIcon, 
  BellIcon, 
  GlobeAltIcon,
  KeyIcon,
  CreditCardIcon 
} from '@heroicons/react/24/outline';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

const settingsSections = [
  {
    id: 'profile',
    name: 'Profile Settings',
    description: 'Manage your account information and preferences',
    icon: UserIcon,
  },
  {
    id: 'security',
    name: 'Security & Privacy',
    description: 'Configure security settings and privacy controls',
    icon: ShieldCheckIcon,
  },
  {
    id: 'notifications',
    name: 'Notifications',
    description: 'Customize your notification preferences',
    icon: BellIcon,
  },
  {
    id: 'api',
    name: 'API & Integrations',
    description: 'Manage API keys and third-party integrations',
    icon: KeyIcon,
  },
  {
    id: 'billing',
    name: 'Billing & Subscription',
    description: 'View billing information and manage subscriptions',
    icon: CreditCardIcon,
  },
  {
    id: 'system',
    name: 'System Settings',
    description: 'Configure system-wide settings and preferences',
    icon: GlobeAltIcon,
  },
];

export function SettingsPage() {
  const [activeSection, setActiveSection] = useState('profile');

  return (
    <div className="space-y-8">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-foreground">Settings</h1>
        <p className="mt-2 text-muted-foreground">
          Manage your account settings and system configuration
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Settings navigation */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Settings</CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <nav className="space-y-1">
                {settingsSections.map((section) => (
                  <button
                    key={section.id}
                    onClick={() => setActiveSection(section.id)}
                    className={`w-full flex items-center space-x-3 px-4 py-3 text-left text-sm transition-colors ${
                      activeSection === section.id
                        ? 'bg-primary text-primary-foreground'
                        : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                    }`}
                  >
                    <section.icon className="h-5 w-5" />
                    <div>
                      <p className="font-medium">{section.name}</p>
                      <p className="text-xs opacity-75">{section.description}</p>
                    </div>
                  </button>
                ))}
              </nav>
            </CardContent>
          </Card>
        </div>

        {/* Settings content */}
        <div className="lg:col-span-3">
          {activeSection === 'profile' && <ProfileSettings />}
          {activeSection === 'security' && <SecuritySettings />}
          {activeSection === 'notifications' && <NotificationSettings />}
          {activeSection === 'api' && <ApiSettings />}
          {activeSection === 'billing' && <BillingSettings />}
          {activeSection === 'system' && <SystemSettings />}
        </div>
      </div>
    </div>
  );
}

function ProfileSettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Profile Information</CardTitle>
          <p className="text-sm text-muted-foreground">
            Update your account profile information
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="form-label">First Name</label>
              <input type="text" className="form-input" defaultValue="Admin" />
            </div>
            <div>
              <label className="form-label">Last Name</label>
              <input type="text" className="form-input" defaultValue="User" />
            </div>
          </div>
          <div>
            <label className="form-label">Email Address</label>
            <input type="email" className="form-input" defaultValue="admin@rwa-platform.com" />
          </div>
          <div>
            <label className="form-label">Bio</label>
            <textarea className="form-input min-h-[100px]" placeholder="Tell us about yourself..." />
          </div>
          <Button className="btn-primary">Save Changes</Button>
        </CardContent>
      </Card>
    </div>
  );
}

function SecuritySettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Security Settings</CardTitle>
          <p className="text-sm text-muted-foreground">
            Manage your account security and authentication
          </p>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-medium">Two-Factor Authentication</h4>
              <p className="text-sm text-muted-foreground">Add an extra layer of security</p>
            </div>
            <Badge className="bg-success/10 text-success">Enabled</Badge>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-medium">Login Notifications</h4>
              <p className="text-sm text-muted-foreground">Get notified of new sign-ins</p>
            </div>
            <Button variant="outline" size="sm">Configure</Button>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-medium">Active Sessions</h4>
              <p className="text-sm text-muted-foreground">Manage your active sessions</p>
            </div>
            <Button variant="outline" size="sm">View Sessions</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function NotificationSettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Notification Preferences</CardTitle>
          <p className="text-sm text-muted-foreground">
            Choose how you want to be notified
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          {[
            'Asset tokenization updates',
            'Transaction confirmations',
            'AI service completions',
            'Security alerts',
            'System maintenance',
          ].map((notification) => (
            <div key={notification} className="flex items-center justify-between">
              <span className="text-sm">{notification}</span>
              <div className="flex space-x-2">
                <Badge variant="secondary" className="text-xs">Email</Badge>
                <Badge variant="secondary" className="text-xs">Push</Badge>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}

function ApiSettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>API Keys</CardTitle>
          <p className="text-sm text-muted-foreground">
            Manage your API keys and access tokens
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h4 className="font-medium">Production API Key</h4>
              <p className="text-sm text-muted-foreground font-mono">rwa_prod_••••••••••••••••</p>
            </div>
            <Button variant="outline" size="sm">Regenerate</Button>
          </div>
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h4 className="font-medium">Development API Key</h4>
              <p className="text-sm text-muted-foreground font-mono">rwa_dev_••••••••••••••••</p>
            </div>
            <Button variant="outline" size="sm">Regenerate</Button>
          </div>
          <Button className="btn-primary">Create New Key</Button>
        </CardContent>
      </Card>
    </div>
  );
}

function BillingSettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Subscription Plan</CardTitle>
          <p className="text-sm text-muted-foreground">
            Manage your subscription and billing information
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-medium">Enterprise Plan</h4>
              <p className="text-sm text-muted-foreground">$299/month • Unlimited assets</p>
            </div>
            <Badge className="bg-primary/10 text-primary">Active</Badge>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 pt-4">
            <div className="text-center">
              <p className="text-2xl font-bold">∞</p>
              <p className="text-sm text-muted-foreground">Assets</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold">100K</p>
              <p className="text-sm text-muted-foreground">API Calls/month</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold">24/7</p>
              <p className="text-sm text-muted-foreground">Support</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function SystemSettings() {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>System Configuration</CardTitle>
          <p className="text-sm text-muted-foreground">
            Configure system-wide settings and preferences
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="form-label">Default Currency</label>
              <select className="form-input">
                <option>USD</option>
                <option>EUR</option>
                <option>GBP</option>
              </select>
            </div>
            <div>
              <label className="form-label">Timezone</label>
              <select className="form-input">
                <option>UTC</option>
                <option>EST</option>
                <option>PST</option>
              </select>
            </div>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-medium">Maintenance Mode</h4>
              <p className="text-sm text-muted-foreground">Enable system maintenance mode</p>
            </div>
            <Button variant="outline" size="sm">Configure</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
