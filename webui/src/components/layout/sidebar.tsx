'use client';

import { Fragment } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Dialog, Transition } from '@headlessui/react';
import {
  HomeIcon,
  CubeIcon,
  CpuChipIcon,
  UsersIcon,
  CreditCardIcon,
  ChartBarIcon,
  Cog6ToothIcon,
  XMarkIcon,
  ShieldCheckIcon,
  DocumentTextIcon,
  BanknotesIcon,
  ClipboardDocumentListIcon,
} from '@heroicons/react/24/outline';
import { cn } from '@/lib/utils';

const navigation = [
  {
    name: 'Dashboard',
    href: '/',
    icon: HomeIcon,
    description: 'Overview and key metrics'
  },
  {
    name: 'Assets',
    href: '/assets',
    icon: CubeIcon,
    description: 'Manage tokenized assets'
  },
  {
    name: 'AI Services',
    href: '/ai',
    icon: CpuChipIcon,
    description: 'AI-powered valuations'
  },
  {
    name: 'Users',
    href: '/users',
    icon: UsersIcon,
    description: 'User management'
  },
  {
    name: 'Trading',
    href: '/trading',
    icon: BanknotesIcon,
    description: 'Trade tokenized assets'
  },
  {
    name: 'Transactions',
    href: '/transactions',
    icon: CreditCardIcon,
    description: 'Transaction history'
  },
  {
    name: 'Analytics',
    href: '/analytics',
    icon: ChartBarIcon,
    description: 'Performance analytics'
  },
  {
    name: 'Compliance',
    href: '/compliance',
    icon: ShieldCheckIcon,
    description: 'Audit and compliance'
  },
  {
    name: 'Reports',
    href: '/reports',
    icon: ClipboardDocumentListIcon,
    description: 'Generate reports'
  },
  {
    name: 'Documentation',
    href: '/docs',
    icon: DocumentTextIcon,
    description: 'API documentation'
  },
  {
    name: 'Settings',
    href: '/settings',
    icon: Cog6ToothIcon,
    description: 'System configuration'
  },
];

interface SidebarProps {
  open: boolean;
  onClose: () => void;
}

export function Sidebar({ open, onClose }: SidebarProps) {
  const pathname = usePathname();

  const SidebarContent = () => (
    <div className="flex h-full flex-col">
      {/* Logo */}
      <div className="flex h-16 shrink-0 items-center px-6">
        <div className="flex items-center space-x-3">
          <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-orange-500 to-red-500 flex items-center justify-center">
            <CubeIcon className="h-5 w-5 text-white" />
          </div>
          <div>
            <h1 className="text-lg font-bold gradient-text">StableRWA</h1>
            <p className="text-xs text-muted-foreground">Asset SDK Platform</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col px-6 py-4">
        <ul role="list" className="flex flex-1 flex-col gap-y-7">
          <li>
            <ul role="list" className="-mx-2 space-y-1">
              {navigation.map((item) => {
                const isActive = pathname === item.href;
                return (
                  <li key={item.name}>
                    <Link
                      href={item.href}
                      className={cn(
                        'group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-medium transition-all duration-200',
                        isActive
                          ? 'bg-primary text-primary-foreground shadow-md'
                          : 'text-foreground hover:text-primary hover:bg-muted'
                      )}
                      onClick={onClose}
                    >
                      <item.icon
                        className={cn(
                          'h-6 w-6 shrink-0 transition-colors',
                          isActive ? 'text-primary-foreground' : 'text-muted-foreground group-hover:text-primary'
                        )}
                        aria-hidden="true"
                      />
                      {item.name}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </li>

          {/* Status indicator */}
          <li className="mt-auto">
            <div className="rounded-lg bg-card p-4 border">
              <div className="flex items-center space-x-3">
                <div className="h-2 w-2 rounded-full bg-success animate-pulse" />
                <div>
                  <p className="text-sm font-medium">System Status</p>
                  <p className="text-xs text-muted-foreground">All services operational</p>
                </div>
              </div>
            </div>
          </li>
        </ul>
      </nav>
    </div>
  );

  return (
    <>
      {/* Mobile sidebar */}
      <Transition.Root show={open} as={Fragment}>
        <Dialog as="div" className="relative z-50 lg:hidden" onClose={onClose}>
          <Transition.Child
            as={Fragment}
            enter="transition-opacity ease-linear duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="transition-opacity ease-linear duration-300"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-background/80 backdrop-blur-sm" />
          </Transition.Child>

          <div className="fixed inset-0 flex">
            <Transition.Child
              as={Fragment}
              enter="transition ease-in-out duration-300 transform"
              enterFrom="-translate-x-full"
              enterTo="translate-x-0"
              leave="transition ease-in-out duration-300 transform"
              leaveFrom="translate-x-0"
              leaveTo="-translate-x-full"
            >
              <Dialog.Panel className="relative mr-16 flex w-full max-w-xs flex-1">
                <div className="absolute left-full top-0 flex w-16 justify-center pt-5">
                  <button
                    type="button"
                    className="-m-2.5 p-2.5 text-foreground hover:text-primary transition-colors"
                    onClick={onClose}
                  >
                    <span className="sr-only">Close sidebar</span>
                    <XMarkIcon className="h-6 w-6" aria-hidden="true" />
                  </button>
                </div>
                <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-card border-r shadow-xl">
                  <SidebarContent />
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </Dialog>
      </Transition.Root>

      {/* Desktop sidebar */}
      <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
        <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-card border-r shadow-sm">
          <SidebarContent />
        </div>
      </div>
    </>
  );
}
