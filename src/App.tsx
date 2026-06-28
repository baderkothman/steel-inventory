import { Navigate, Route, Routes } from "react-router-dom";

import { AppLayout } from "./app/layout/AppLayout";
import { LoadingState } from "./components/feedback/PageState";
import { AuthPage } from "./features/auth/AuthPage";
import { useAuth } from "./features/auth/AuthContext";
import { BackupPage } from "./features/backup/BackupPage";
import { CategoriesPage } from "./features/categories/CategoriesPage";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { ExpensesPage } from "./features/expenses/ExpensesPage";
import { PurchasesPage, SalesPage } from "./features/invoices/InvoicePages";
import { PaymentsPage } from "./features/payments/PaymentsPage";
import { CustomersPage, SuppliersPage } from "./features/parties/PartiesPage";
import { ProductsPage } from "./features/products/ProductsPage";
import { ReportsPage } from "./features/reports/ReportsPage";
import { SettingsPage } from "./features/settings/SettingsPage";

export default function App() {
  const { admin, hasAdmin, loading } = useAuth();

  if (loading || hasAdmin === null) {
    return <LoadingState label="Starting Steel Inventory" />;
  }

  if (!hasAdmin || !admin) {
    return <AuthPage />;
  }

  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<DashboardPage />} />
        <Route path="products" element={<ProductsPage />} />
        <Route path="categories" element={<CategoriesPage />} />
        <Route path="suppliers" element={<SuppliersPage />} />
        <Route path="customers" element={<CustomersPage />} />
        <Route path="purchases" element={<PurchasesPage />} />
        <Route path="sales" element={<SalesPage />} />
        <Route path="expenses" element={<ExpensesPage />} />
        <Route path="payments" element={<PaymentsPage />} />
        <Route path="reports" element={<ReportsPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="backup" element={<BackupPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
