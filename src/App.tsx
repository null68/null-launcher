import { useState } from "react";
import { Sidebar } from "./components/Sidebar";
import "./styles/tokens.css";
import "./styles/global.css";
import "./styles/components.css";
import { InstancesView } from "./views/InstancesView";
import { ScreenshotsView } from "./views/ScreenshotsView";

export type ViewId = "instances" | "screenshots" | "settings";

function App() {
  const [view, setView] = useState<ViewId>("instances");

  return (
    <div className="app">
      <Sidebar active={view} onSelect={setView} />
      <main className="main">
        {view == "instances" && <InstancesView />}
        {view == "screenshots" && <ScreenshotsView />}
      </main>
    </div>
  );
}

export default App;
