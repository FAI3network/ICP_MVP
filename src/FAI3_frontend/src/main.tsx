import { StrictMode, useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import './index.scss';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Leaderboard, Model, Whoami } from './pages';
import { Layout, Providers } from './components';
import { IDL as IDLType } from "@dfinity/candid";
import { AuthClientContext, DataContext } from './utils';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Actor, ActorSubclass, ActorMethod } from '@dfinity/agent';
import { idlFactory, canisterId } from '../../declarations/FAI3_backend';
import { Toaster } from "@/components/ui/sonner";

function App() {
  return (<StrictMode>
    <Providers>
      <Router>
        <Routes>
          <Route path="/" element={<Layout />} >
            <Route index element={<Leaderboard />} />
            <Route path="model/:modelId" element={<Model />} />
            <Route path="whoami" element={<Whoami />} />
          </Route>
        </Routes>
      </Router>
      <Toaster />
    </Providers>
  </StrictMode>
  )
}

const rootElement = document.getElementById('root');
if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <App />
  );
}