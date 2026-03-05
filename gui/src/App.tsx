import { ThemeProvider } from "@/providers/ThemeProvider";
import { TitleBar } from "@/components/layout/TitleBar";
import { Sidebar } from "@/components/layout/Sidebar";
import { Routes, Route } from "react-router-dom";

import HomePage from "@/pages/HomePage";
import PluginPage from "@/pages/PluginPage";
import SettingsPage from "@/pages/SettingsPage";
import LogsPage from "./pages/LogsPage";
import NoPluginsFoundPage from "./pages/NoPluginFoundPage";
import { ThemeInitializer } from "./components/layout/ModeToggle";
import { PluginsProvider } from "./providers/PluginsProvider";

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <ThemeInitializer />
      <div className="bg-sidebar flex h-screen flex-col">
        <TitleBar />
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />

          <PluginsProvider>
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/dashboard" element={<HomePage />} />

              <Route path="/log" element={<LogsPage />} />
              <Route path="/settings" element={<SettingsPage />} />
              <Route path="/no-plugins" element={<NoPluginsFoundPage />} />

              <Route path="/plugin/:pid" element={<PluginPage />} />
            </Routes>
          </PluginsProvider>
        </div>
      </div>
    </ThemeProvider>
  );
}
