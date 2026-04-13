import {
  LayoutDashboard,
  BookOpen,
  FileText,
  ShoppingCart,
  Truck,
  Package,
  Receipt,
  Building2,
  Landmark,
  BarChart3,
  Settings,
  FileSpreadsheet,
  Brain,
  FolderOpen,
  LucideIcon,
} from "lucide-react";

export interface NavItem {
  label: string;
  path: string;
  icon: LucideIcon;
  requiredPermission: string;
}

export const NAV_ITEMS: NavItem[] = [
  { label: "Dashboard", path: "/dashboard", icon: LayoutDashboard, requiredPermission: "dashboard:view" },
  { label: "Accounting", path: "/accounting", icon: BookOpen, requiredPermission: "accounting:view" },
  { label: "Journal Entries", path: "/journal-entries", icon: FileText, requiredPermission: "journal:view" },
  { label: "Sales", path: "/sales", icon: ShoppingCart, requiredPermission: "sales:view" },
  { label: "Purchases", path: "/purchases", icon: Truck, requiredPermission: "purchases:view" },
  { label: "Inventory", path: "/inventory", icon: Package, requiredPermission: "inventory:view" },
  { label: "Tax", path: "/tax", icon: Receipt, requiredPermission: "tax:view" },
  { label: "Fixed Assets", path: "/fixed-assets", icon: Building2, requiredPermission: "assets:view" },
  { label: "Loans", path: "/loans", icon: Landmark, requiredPermission: "loans:view" },
  { label: "Reports", path: "/reports", icon: BarChart3, requiredPermission: "reports:view" },
  { label: "Admin", path: "/admin", icon: Settings, requiredPermission: "admin:view" },
  { label: "E-Invoicing", path: "/einvoicing", icon: FileSpreadsheet, requiredPermission: "einvoicing:view" },
  { label: "AI Analysis", path: "/ai-analysis", icon: Brain, requiredPermission: "ai:view" },
  { label: "Documents", path: "/documents", icon: FolderOpen, requiredPermission: "documents:view" },
];
