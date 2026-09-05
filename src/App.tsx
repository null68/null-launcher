import { useEffect, useState } from "react";
import { emit } from "@tauri-apps/api/event";
import { Sidebar } from "./components/Sidebar";
import "./styles/tokens.css";
import "./styles/global.css";
import "./styles/components.css";
import { InstancesView } from "./views/InstancesView";
import { ScreenshotsView } from "./views/ScreenshotsView";
import { SettingsView } from "./views/SettingsView";
import { useInstallManager } from "./hooks/useInstallManager";

export type ViewId = "instances" | "screenshots" | "settings";

function App() {
  const [view, setView] = useState<ViewId>("instances");
  const installManager = useInstallManager();

  useEffect(() => {
    emit("app-ready");
  }, []);

  return (
    <div className="app">
      <Sidebar active={view} onSelect={setView} installing={installManager.installing} />
      <main className="main">
        {view == "instances" && <InstancesView installManager={installManager} />}
        {view == "screenshots" && <ScreenshotsView />}
        {view == "settings" && <SettingsView />}
      </main>
    </div>
  );
}

export default App;
