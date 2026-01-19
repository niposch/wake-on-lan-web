import React, { createContext, useContext, useState, useEffect } from 'react';
import type { User, LoginRequest } from '../types';
import { api } from '../services';

interface AuthContextType {
    user: User | null;
    isLoading: boolean;
    login: (data: LoginRequest) => Promise<void>;
    logout: () => Promise<void>;
    refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        refreshUser().finally(() => setIsLoading(false));

        const handleUnauthorized = () => {
            setUser(null);
        };

        window.addEventListener('auth:unauthorized', handleUnauthorized);
        return () => window.removeEventListener('auth:unauthorized', handleUnauthorized);
    }, []);

    const refreshUser = async () => {
        try {
            const user = await api.getMe();
            setUser(user);
        } catch (error) {
            localStorage.removeItem('access_token');
            localStorage.removeItem('refresh_token');
            setUser(null);
            // Ignore error, maybe token expired or not logged in
        }
    };

    const login = async (data: LoginRequest) => {
        const response = await api.login(data);
        setUser(response.user);
    };

    const logout = async () => {
        await api.logout();
        setUser(null);
    };

    return (
        <AuthContext.Provider value={{ user, isLoading, login, logout, refreshUser }}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
}
