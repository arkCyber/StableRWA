# =====================================================================================
# File: infrastructure/terraform/variables.tf
# Description: Terraform variables for StableRWA platform infrastructure
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# General Configuration
variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "stablerwa"
}

variable "environment" {
  description = "Environment name (dev, staging, production)"
  type        = string
  validation {
    condition     = contains(["dev", "staging", "production"], var.environment)
    error_message = "Environment must be one of: dev, staging, production."
  }
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "domain_name" {
  description = "Domain name for the application"
  type        = string
  default     = "stablerwa.com"
}

# Kubernetes Configuration
variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "node_groups" {
  description = "EKS node groups configuration"
  type = map(object({
    instance_types = list(string)
    capacity_type  = string
    min_size       = number
    max_size       = number
    desired_size   = number
    disk_size      = number
    disk_type      = string
    labels         = map(string)
    taints = list(object({
      key    = string
      value  = string
      effect = string
    }))
  }))
  default = {
    general = {
      instance_types = ["t3.medium"]
      capacity_type  = "ON_DEMAND"
      min_size       = 2
      max_size       = 10
      desired_size   = 3
      disk_size      = 50
      disk_type      = "gp3"
      labels = {
        role = "general"
      }
      taints = []
    }
    compute = {
      instance_types = ["c5.large"]
      capacity_type  = "SPOT"
      min_size       = 0
      max_size       = 5
      desired_size   = 1
      disk_size      = 100
      disk_type      = "gp3"
      labels = {
        role = "compute"
      }
      taints = [
        {
          key    = "compute"
          value  = "true"
          effect = "NO_SCHEDULE"
        }
      ]
    }
  }
}

# Database Configuration
variable "db_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "db_name" {
  description = "Database name"
  type        = string
  default     = "stablerwa"
}

variable "db_username" {
  description = "Database username"
  type        = string
  default     = "stablerwa"
}

variable "db_allocated_storage" {
  description = "Initial allocated storage for RDS instance (GB)"
  type        = number
  default     = 100
}

variable "db_max_allocated_storage" {
  description = "Maximum allocated storage for RDS instance (GB)"
  type        = number
  default     = 1000
}

variable "db_backup_retention_period" {
  description = "Database backup retention period in days"
  type        = number
  default     = 7
}

variable "db_multi_az" {
  description = "Enable Multi-AZ deployment for RDS"
  type        = bool
  default     = false
}

variable "db_deletion_protection" {
  description = "Enable deletion protection for RDS"
  type        = bool
  default     = false
}

# Redis Configuration
variable "redis_node_type" {
  description = "ElastiCache Redis node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_num_cache_nodes" {
  description = "Number of cache nodes in the Redis cluster"
  type        = number
  default     = 1
}

variable "redis_parameter_group_name" {
  description = "Redis parameter group name"
  type        = string
  default     = "default.redis7"
}

variable "redis_engine_version" {
  description = "Redis engine version"
  type        = string
  default     = "7.0"
}

variable "redis_snapshot_retention_limit" {
  description = "Number of days to retain Redis snapshots"
  type        = number
  default     = 0
}

# Monitoring Configuration
variable "enable_monitoring" {
  description = "Enable enhanced monitoring"
  type        = bool
  default     = true
}

variable "monitoring_interval" {
  description = "Monitoring interval in seconds"
  type        = number
  default     = 60
}

variable "performance_insights_enabled" {
  description = "Enable Performance Insights"
  type        = bool
  default     = true
}

variable "performance_insights_retention_period" {
  description = "Performance Insights retention period in days"
  type        = number
  default     = 7
}

# Security Configuration
variable "enable_flow_logs" {
  description = "Enable VPC Flow Logs"
  type        = bool
  default     = true
}

variable "enable_encryption" {
  description = "Enable encryption at rest"
  type        = bool
  default     = true
}

variable "kms_key_id" {
  description = "KMS key ID for encryption"
  type        = string
  default     = null
}

# Backup Configuration
variable "backup_window" {
  description = "Database backup window"
  type        = string
  default     = "03:00-06:00"
}

variable "maintenance_window" {
  description = "Database maintenance window"
  type        = string
  default     = "Mon:00:00-Mon:03:00"
}

# Scaling Configuration
variable "enable_autoscaling" {
  description = "Enable cluster autoscaling"
  type        = bool
  default     = true
}

variable "autoscaling_min_capacity" {
  description = "Minimum capacity for autoscaling"
  type        = number
  default     = 2
}

variable "autoscaling_max_capacity" {
  description = "Maximum capacity for autoscaling"
  type        = number
  default     = 100
}

variable "autoscaling_target_cpu" {
  description = "Target CPU utilization for autoscaling"
  type        = number
  default     = 70
}

# Load Balancer Configuration
variable "enable_alb" {
  description = "Enable Application Load Balancer"
  type        = bool
  default     = true
}

variable "alb_idle_timeout" {
  description = "ALB idle timeout in seconds"
  type        = number
  default     = 60
}

variable "enable_deletion_protection" {
  description = "Enable ALB deletion protection"
  type        = bool
  default     = false
}

# SSL/TLS Configuration
variable "ssl_policy" {
  description = "SSL policy for HTTPS listeners"
  type        = string
  default     = "ELBSecurityPolicy-TLS-1-2-2017-01"
}

variable "certificate_arn" {
  description = "ACM certificate ARN"
  type        = string
  default     = null
}

# Logging Configuration
variable "enable_cloudwatch_logs" {
  description = "Enable CloudWatch logs"
  type        = bool
  default     = true
}

variable "log_retention_days" {
  description = "CloudWatch logs retention period in days"
  type        = number
  default     = 30
}

# Cost Optimization
variable "use_spot_instances" {
  description = "Use spot instances for cost optimization"
  type        = bool
  default     = false
}

variable "spot_instance_types" {
  description = "Instance types for spot instances"
  type        = list(string)
  default     = ["t3.medium", "t3.large", "m5.large"]
}

# Disaster Recovery
variable "enable_cross_region_backup" {
  description = "Enable cross-region backup"
  type        = bool
  default     = false
}

variable "backup_region" {
  description = "Backup region for disaster recovery"
  type        = string
  default     = "us-east-1"
}

# Compliance
variable "enable_compliance_monitoring" {
  description = "Enable compliance monitoring"
  type        = bool
  default     = true
}

variable "compliance_standards" {
  description = "Compliance standards to monitor"
  type        = list(string)
  default     = ["SOC2", "PCI-DSS", "GDPR"]
}

# Environment-specific overrides
variable "environment_config" {
  description = "Environment-specific configuration overrides"
  type = map(object({
    db_instance_class                = optional(string)
    redis_node_type                  = optional(string)
    enable_multi_az                  = optional(bool)
    backup_retention_period          = optional(number)
    enable_deletion_protection       = optional(bool)
    enable_performance_insights      = optional(bool)
    min_capacity                     = optional(number)
    max_capacity                     = optional(number)
    desired_capacity                 = optional(number)
  }))
  default = {
    dev = {
      db_instance_class           = "db.t3.micro"
      redis_node_type            = "cache.t3.micro"
      enable_multi_az            = false
      backup_retention_period    = 1
      enable_deletion_protection = false
      enable_performance_insights = false
      min_capacity               = 1
      max_capacity               = 3
      desired_capacity           = 1
    }
    staging = {
      db_instance_class           = "db.t3.small"
      redis_node_type            = "cache.t3.small"
      enable_multi_az            = false
      backup_retention_period    = 7
      enable_deletion_protection = false
      enable_performance_insights = true
      min_capacity               = 2
      max_capacity               = 5
      desired_capacity           = 2
    }
    production = {
      db_instance_class           = "db.r5.large"
      redis_node_type            = "cache.r5.large"
      enable_multi_az            = true
      backup_retention_period    = 30
      enable_deletion_protection = true
      enable_performance_insights = true
      min_capacity               = 3
      max_capacity               = 20
      desired_capacity           = 5
    }
  }
}

# Tags
variable "additional_tags" {
  description = "Additional tags to apply to all resources"
  type        = map(string)
  default     = {}
}
