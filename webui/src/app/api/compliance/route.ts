import { NextRequest, NextResponse } from 'next/server';

// Mock compliance data
const mockComplianceData = {
  metrics: {
    kycCompletion: {
      value: '98.5%',
      target: '95%',
      status: 'excellent'
    },
    amlScreening: {
      value: '100%',
      target: '100%',
      status: 'excellent'
    },
    regulatoryFilings: {
      value: '92%',
      target: '95%',
      status: 'warning'
    },
    auditCompliance: {
      value: '96%',
      target: '90%',
      status: 'excellent'
    }
  },
  overallStatus: {
    status: 'compliant',
    score: 96.5,
    lastCheck: new Date().toISOString(),
    issues: [
      {
        type: 'documentation',
        description: 'Missing documentation for 3 assets',
        severity: 'medium',
        dueDate: '2024-02-15'
      }
    ]
  },
  alerts: [
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
  ],
  jurisdictions: [
    {
      name: 'United States',
      status: 'compliant',
      lastReview: '2024-01-15',
      requirements: ['SEC Registration', 'FINRA Compliance', 'State Licensing'],
      nextReview: '2024-04-15'
    },
    {
      name: 'European Union',
      status: 'compliant',
      lastReview: '2024-01-10',
      requirements: ['MiFID II', 'GDPR', 'AML5'],
      nextReview: '2024-04-10'
    },
    {
      name: 'United Kingdom',
      status: 'pending_review',
      lastReview: '2023-12-20',
      requirements: ['FCA Authorization', 'MLR 2017', 'CASS Rules'],
      nextReview: '2024-03-20'
    },
    {
      name: 'Singapore',
      status: 'compliant',
      lastReview: '2024-01-05',
      requirements: ['MAS License', 'SFA Compliance', 'PDPA'],
      nextReview: '2024-04-05'
    },
    {
      name: 'Japan',
      status: 'non_compliant',
      lastReview: '2023-11-30',
      requirements: ['FSA Registration', 'FIEA Compliance', 'AML/CFT'],
      nextReview: '2024-02-29',
      issues: ['Missing FSA registration', 'Incomplete AML procedures']
    }
  ],
  reports: [
    {
      id: 1,
      name: 'KYC Summary Report',
      type: 'kyc_summary',
      description: 'Customer verification status summary',
      frequency: 'Monthly',
      lastGenerated: '2024-01-01',
      nextDue: '2024-02-01',
      status: 'current'
    },
    {
      id: 2,
      name: 'AML Screening Report',
      type: 'aml_screening',
      description: 'Anti-money laundering screening results',
      frequency: 'Weekly',
      lastGenerated: '2024-01-14',
      nextDue: '2024-01-21',
      status: 'current'
    },
    {
      id: 3,
      name: 'Regulatory Filing Report',
      type: 'regulatory_filing',
      description: 'Status of regulatory submissions',
      frequency: 'Quarterly',
      lastGenerated: '2023-12-31',
      nextDue: '2024-03-31',
      status: 'upcoming'
    },
    {
      id: 4,
      name: 'Audit Trail Report',
      type: 'audit_trail',
      description: 'Complete audit trail documentation',
      frequency: 'On-demand',
      lastGenerated: '2024-01-10',
      nextDue: null,
      status: 'available'
    }
  ]
};

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const section = searchParams.get('section') || 'overview';
    const jurisdiction = searchParams.get('jurisdiction');

    let responseData = { ...mockComplianceData };

    // Filter by jurisdiction if specified
    if (jurisdiction) {
      responseData.jurisdictions = responseData.jurisdictions.filter(
        j => j.name.toLowerCase().includes(jurisdiction.toLowerCase())
      );
    }

    // Return specific section data
    switch (section) {
      case 'alerts':
        return NextResponse.json({ alerts: responseData.alerts });
      case 'jurisdictions':
        return NextResponse.json({ jurisdictions: responseData.jurisdictions });
      case 'reports':
        return NextResponse.json({ reports: responseData.reports });
      default:
        return NextResponse.json(responseData);
    }
  } catch (error) {
    console.error('Compliance API error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch compliance data' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { action, data } = body;

    switch (action) {
      case 'run_compliance_check':
        // Mock compliance check execution
        const checkResult = {
          checkId: `check_${Date.now()}`,
          status: 'completed',
          results: {
            kycCompletion: Math.floor(Math.random() * 5) + 95,
            amlScreening: 100,
            regulatoryFilings: Math.floor(Math.random() * 10) + 90,
            auditCompliance: Math.floor(Math.random() * 8) + 92
          },
          issues: Math.random() > 0.7 ? [
            {
              type: 'documentation',
              description: 'Minor documentation gaps detected',
              severity: 'low'
            }
          ] : [],
          executionTime: Math.floor(Math.random() * 5000) + 1000,
          timestamp: new Date().toISOString()
        };
        return NextResponse.json(checkResult);

      case 'generate_report':
        // Mock report generation
        const reportResult = {
          reportId: `report_${Date.now()}`,
          type: data.reportType || 'compliance_summary',
          status: 'generating',
          estimatedCompletion: new Date(Date.now() + 30000).toISOString(),
          downloadUrl: null
        };
        
        // Simulate async report generation
        setTimeout(() => {
          reportResult.status = 'completed';
          reportResult.downloadUrl = `/api/compliance/reports/${reportResult.reportId}/download`;
        }, 5000);

        return NextResponse.json(reportResult);

      case 'update_alert_status':
        // Mock alert status update
        const alertUpdate = {
          alertId: data.alertId,
          oldStatus: data.oldStatus,
          newStatus: data.newStatus,
          updatedAt: new Date().toISOString(),
          updatedBy: 'system'
        };
        return NextResponse.json(alertUpdate);

      default:
        return NextResponse.json(
          { error: 'Unknown action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Compliance action error:', error);
    return NextResponse.json(
      { error: 'Failed to execute compliance action' },
      { status: 500 }
    );
  }
}
