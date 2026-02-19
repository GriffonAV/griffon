import { ThemeProvider } from "@/components/theme-provider";
import { TitleBar } from "@/components/title-bar";
import { Sidebar } from "@/components/sidebar";
import { Routes, Route } from "react-router-dom";

import HomePage from "@/pages/home/HomePage";
import PluginPage from "@/pages/plugins/PluginPage";
import SettingsPage from "@/pages/settings/SettingsPage";
import HistoryPage from "./pages/log/HistoryPage";

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="bg-sidebar flex h-screen flex-col">
        <TitleBar />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />

          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/dashboard" element={<HomePage />} />

            <Route path="/log" element={<HistoryPage />} />
            <Route path="/plugin/:pid" element={<PluginPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>

        </div>
      </div>
    </ThemeProvider>
  );
}
