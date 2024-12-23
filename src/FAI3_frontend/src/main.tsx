import { StrictMode, useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import './index.scss';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Leaderboard, Model } from './pages';
import { Layout, Providers } from './components';
import { IDL as IDLType } from "@dfinity/candid";
import { AuthClientContext, DataContext } from './utils';
import { AuthClient } from "@dfinity/auth-client";
import { HttpAgent, Actor, ActorSubclass, ActorMethod } from '@dfinity/agent';
import { idlFactory, canisterId } from '../../declarations/FAI3_backend';

function App() {
  return (<StrictMode>
    <Providers>
      <Router>
        <Routes>
          <Route path="/" element={<Layout />} >
            <Route index element={<Leaderboard />} />
            <Route path="model/:modelId" element={<Model />} />
          </Route>
        </Routes>
      </Router>
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