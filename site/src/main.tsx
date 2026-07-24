import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import NotePage from "./NotePage";
import { Nav, useRoute } from "./Nav";
import "./index.css";

function Root() {
  const path = useRoute();
  const onNote = path.startsWith("/technical-note");
  return (
    <div className="min-h-screen">
      <Nav path={path} />
      {onNote ? <NotePage /> : <App />}
    </div>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Root />
  </StrictMode>
);
