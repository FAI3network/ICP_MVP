import { useState, createContext, forwardRef } from 'react';
import { CircleCheck, CircleAlert } from 'lucide-react';
import { cn } from "../../utils"

interface ToastType {
  title: string;
  message?: string;
  type?: "success" | "error" | "warning" | "info";
  duration?: number;
  onStatusChange?: ({status} : {status: "visible" | "closing" | "hidden"}) => void;
  max?: number;
  placement?: "top" | "bottom" | "top-left" | "top-right" | "bottom-left" | "bottom-right";
  offset?: number;
}

