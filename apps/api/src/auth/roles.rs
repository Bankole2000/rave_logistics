//! Staff roles and what each may do. The permissions come from the staff rows of the coverage
//! matrix in docs/FEATURES.md (user + action + resource); add one here when a feature needs it.
//! Every check happens on the server; clients use the list only to show the right tools.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// An internal staff role (R-22; users U-06 to U-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Support,
    Operations,
    Admin,
    Hr,
    Finance,
    Management,
}

impl Role {
    pub const ALL: [Self; 6] = [
        Self::Support,
        Self::Operations,
        Self::Admin,
        Self::Hr,
        Self::Finance,
        Self::Management,
    ];

    /// The value stored in `staff_members.role` and `staff_invitations.role`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Support => "support",
            Self::Operations => "operations",
            Self::Admin => "admin",
            Self::Hr => "hr",
            Self::Finance => "finance",
            Self::Management => "management",
        }
    }

    pub fn from_stored(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == value)
    }

    /// A name for messages, e.g. "an operations manager".
    pub fn label(self) -> &'static str {
        match self {
            Self::Support => "a support agent",
            Self::Operations => "an operations manager",
            Self::Admin => "an admin",
            Self::Hr => "an HR officer",
            Self::Finance => "a finance officer",
            Self::Management => "management",
        }
    }

    pub fn permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            // U-06 Support agent
            Self::Support => &[
                VendorsApprove,
                OrdersSearch,
                OrdersCancel,
                DriversApprove,
                DriversDecline,
                PaymentsRefund,
                TicketsEdit,
                MessagesCreate,
            ],
            // U-07 Operations manager
            Self::Operations => &[
                StoresSuspend,
                DeliveriesTrack,
                DeliveriesAssign,
                DriversSuspend,
            ],
            // U-08 Admin
            Self::Admin => &[
                ZonesCreate,
                ZonesEdit,
                StaffView,
                StaffCreate,
                StaffSuspend,
                PartnersCreate,
                AuditView,
                PromotionsCreate,
            ],
            // U-09 HR officer
            Self::Hr => &[StaffView, StaffEdit],
            // U-10 Finance officer
            Self::Finance => &[PaymentsView, PayoutsApprove, ReportsExport],
            // U-11 Management
            Self::Management => &[ReportsView],
        }
    }

    pub fn can(self, permission: Permission) -> bool {
        self.permissions().contains(&permission)
    }
}

/// Something a staff role may do, named `<resource>.<action>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub enum Permission {
    #[serde(rename = "audit.view")]
    AuditView,
    #[serde(rename = "deliveries.assign")]
    DeliveriesAssign,
    #[serde(rename = "deliveries.track")]
    DeliveriesTrack,
    #[serde(rename = "drivers.approve")]
    DriversApprove,
    #[serde(rename = "drivers.decline")]
    DriversDecline,
    #[serde(rename = "drivers.suspend")]
    DriversSuspend,
    #[serde(rename = "messages.create")]
    MessagesCreate,
    #[serde(rename = "orders.cancel")]
    OrdersCancel,
    #[serde(rename = "orders.search")]
    OrdersSearch,
    #[serde(rename = "partners.create")]
    PartnersCreate,
    #[serde(rename = "payments.refund")]
    PaymentsRefund,
    #[serde(rename = "payments.view")]
    PaymentsView,
    #[serde(rename = "payouts.approve")]
    PayoutsApprove,
    #[serde(rename = "promotions.create")]
    PromotionsCreate,
    #[serde(rename = "reports.export")]
    ReportsExport,
    #[serde(rename = "reports.view")]
    ReportsView,
    /// Invite staff members (FT-072).
    #[serde(rename = "staff.create")]
    StaffCreate,
    /// Edit HR details (FT-074).
    #[serde(rename = "staff.edit")]
    StaffEdit,
    /// Suspend and reinstate staff members (FT-073).
    #[serde(rename = "staff.suspend")]
    StaffSuspend,
    /// List staff members. Not in the matrix; needed by those who invite, suspend or edit staff.
    #[serde(rename = "staff.view")]
    StaffView,
    #[serde(rename = "stores.suspend")]
    StoresSuspend,
    #[serde(rename = "tickets.edit")]
    TicketsEdit,
    #[serde(rename = "vendors.approve")]
    VendorsApprove,
    #[serde(rename = "zones.create")]
    ZonesCreate,
    #[serde(rename = "zones.edit")]
    ZonesEdit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_values_round_trip() {
        for role in Role::ALL {
            assert_eq!(Role::from_stored(role.as_str()), Some(role));
        }
        assert_eq!(Role::from_stored("superuser"), None);
    }

    #[test]
    fn only_admins_manage_staff() {
        for role in Role::ALL {
            let admin = role == Role::Admin;
            assert_eq!(role.can(Permission::StaffCreate), admin, "{role:?}");
            assert_eq!(role.can(Permission::StaffSuspend), admin, "{role:?}");
        }
        assert!(Role::Hr.can(Permission::StaffView));
        assert!(!Role::Support.can(Permission::StaffView));
    }

    #[test]
    fn every_role_has_a_permission() {
        for role in Role::ALL {
            assert!(!role.permissions().is_empty(), "{role:?}");
        }
    }
}
