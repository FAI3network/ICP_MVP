import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';
import './index.scss';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Leaderboard, Model } from './pages';
import { Layout } from './components';
import { IDL as IDLType } from "@dfinity/candid";
import { authClientContext } from './utils';
import { AuthClient } from "@dfinity/auth-client";

const rootElement = document.getElementById('root');
if (rootElement) {
  const authClient = await AuthClient.create();

  ReactDOM.createRoot(rootElement).render(
    <StrictMode>
      <authClientContext.Provider value={authClient}>
        <Router>
          <Routes>
            <Route path="/" element={<Layout />} >
              <Route index element={<Leaderboard />} />
              <Route path="model/:modelId" element={<Model />} />
            </Route>
          </Routes>
        </Router>
      </authClientContext.Provider>
    </StrictMode>
  );
}
