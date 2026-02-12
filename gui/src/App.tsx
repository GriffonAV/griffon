import { ThemeProvider } from "@/components/theme-provider";
import { TitleBar } from "@/components/title-bar";
import { Sidebar } from "@/components/sidebar";
import { Routes, Route } from "react-router-dom";

import HomePage from "@/pages/home/HomePage";
import PluginPage from "@/pages/plugins/PluginPage";
import SettingsPage from "@/pages/settings/SettingsPage";

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="flex h-screen flex-col">
        <TitleBar />
        <div className="flex flex-1">
          <Sidebar />

          <div className="flex-1 border-l p-4">
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/plugin/:pid" element={<PluginPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Routes>
          </div>
        </div>
      </div>
    </ThemeProvider>
  );
}
