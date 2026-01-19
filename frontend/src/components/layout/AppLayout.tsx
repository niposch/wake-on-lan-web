import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';
import { Button } from '../ui/button';
import { LayoutDashboard, Users, Server, LogOut, KeyRound } from 'lucide-react';

export default function AppLayout({ children }: { children: React.ReactNode }) {
    const { user, logout } = useAuth();
    const location = useLocation();

    const isActive = (path: string) => location.pathname === path;
    const isAdmin = user?.role === 'admin';

    if (!isAdmin) {
        return (
            <div className="min-h-screen bg-gray-50">
                <header className="bg-white border-b shadow-sm sticky top-0 z-10">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
                        <Link to="/" className="flex items-center gap-2 font-bold text-xl hover:opacity-80 transition-opacity">
                            <Server className="h-6 w-6" />
                            WoL Manager
                        </Link>
                        
                        <div className="flex items-center gap-4">
                            <span className="text-sm text-gray-500 hidden sm:inline-block">
                                {user?.username}
                            </span>
                            <Link to="/change-password">
                                <Button variant="ghost" size="sm" className="gap-2">
                                    <KeyRound className="h-4 w-4" />
                                    <span className="hidden sm:inline">Password</span>
                                </Button>
                            </Link>
                            <Button 
                                variant="ghost" 
                                size="sm"
                                className="text-red-600 hover:text-red-700 hover:bg-red-50 gap-2"
                                onClick={() => logout()}
                            >
                                <LogOut className="h-4 w-4" />
                                <span className="hidden sm:inline">Logout</span>
                            </Button>
                        </div>
                    </div>
                </header>

                <main className="max-w-7xl mx-auto p-4 sm:p-6 lg:p-8">
                    {children}
                </main>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-50 flex">
            {/* Sidebar */}
            <aside className="w-64 bg-white border-r shadow-sm fixed h-full z-20">
                <div className="p-6">
                    <Link to="/" className="text-xl font-bold flex items-center gap-2 hover:opacity-80 transition-opacity">
                        <Server className="h-6 w-6" />
                        WoL Manager
                    </Link>
                </div>
                <nav className="px-4 space-y-2">
                    <Link to="/">
                        <Button 
                            variant={isActive('/') ? "secondary" : "ghost"} 
                            className="w-full justify-start gap-2"
                        >
                            <LayoutDashboard className="h-4 w-4" />
                            Devices
                        </Button>
                    </Link>
                    
                    <Link to="/users">
                        <Button 
                            variant={isActive('/users') ? "secondary" : "ghost"} 
                            className="w-full justify-start gap-2"
                        >
                            <Users className="h-4 w-4" />
                            Users
                        </Button>
                    </Link>
                </nav>
                <div className="absolute bottom-4 left-0 right-0 px-4">
                    <div className="mb-4 px-2 text-sm text-gray-500">
                        Logged in as <span className="font-semibold">{user?.username}</span>
                    </div>
                    <Link to="/change-password">
                        <Button 
                            variant={isActive('/change-password') ? "secondary" : "ghost"} 
                            className="w-full justify-start gap-2 mb-2"
                        >
                            <KeyRound className="h-4 w-4" />
                            Password
                        </Button>
                    </Link>
                    <Button 
                        variant="outline" 
                        className="w-full justify-start gap-2 text-red-600 hover:text-red-700 hover:bg-red-50"
                        onClick={() => logout()}
                    >
                        <LogOut className="h-4 w-4" />
                        Logout
                    </Button>
                </div>
            </aside>

            {/* Main Content */}
            <div className="ml-64 flex-1">
                <main className="max-w-7xl mx-auto p-4 sm:p-6 lg:p-8">
                    {children}
                </main>
            </div>
        </div>
    );
}
