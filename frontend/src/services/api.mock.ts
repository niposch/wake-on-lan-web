import type { ApiService } from './api.interface';
import type { 
    User, Device, LoginRequest, LoginResponse, CreateUserRequest, CreateDeviceRequest, UpdateDeviceRequest, AdminResetPasswordRequest, AdminResetPasswordResponse 
} from '../types';

const STORAGE_KEY_USERS = 'wol_demo_users';
const STORAGE_KEY_PASSWORDS = 'wol_demo_passwords';
const STORAGE_KEY_DEVICES = 'wol_demo_devices';
const MOCK_ACCESS_TOKEN_KEY = 'demo_access_token';

// Initial Mock Data
const DEFAULT_USERS: User[] = [
    {
        id: 1,
        username: 'admin',
        role: 'admin',
        force_password_change: false,
        is_disabled: false,
        last_login_at: new Date().toISOString()
    },
    {
        id: 2,
        username: 'user',
        role: 'user',
        force_password_change: false,
        is_disabled: false,
        last_login_at: undefined
    }
];

const DEFAULT_PASSWORDS: Record<number, string> = {
    1: 'admin',
    2: 'user'
};

const DEFAULT_DEVICES: Device[] = [
    {
        id: 1,
        name: 'Gaming Desktop',
        mac_address: '00:11:22:33:44:55',
        ip_address: '192.168.1.10',
        broadcast_addr: '192.168.1.255',
        icon: 'desktop',
        is_online: false,
        can_shutdown: false,
        last_seen_at: undefined
    },
    {
        id: 2,
        name: 'Home Server',
        mac_address: 'AA:BB:CC:DD:EE:FF',
        ip_address: '192.168.1.200',
        broadcast_addr: '192.168.1.255',
        icon: 'server',
        is_online: true,
        can_shutdown: false,
        last_seen_at: new Date().toISOString()
    },
    {
        id: 3,
        name: 'Living Room TV',
        mac_address: '11:22:33:44:55:66',
        icon: 'monitor',
        is_online: false,
        can_shutdown: false,
        last_seen_at: undefined
    }
];

// Helpers
const loadUsers = (): User[] => {
    const s = localStorage.getItem(STORAGE_KEY_USERS);
    if (s) return JSON.parse(s);
    localStorage.setItem(STORAGE_KEY_USERS, JSON.stringify(DEFAULT_USERS));
    return DEFAULT_USERS;
};

const saveUsers = (users: User[]) => {
    localStorage.setItem(STORAGE_KEY_USERS, JSON.stringify(users));
};

const loadPasswords = (): Record<number, string> => {
    const s = localStorage.getItem(STORAGE_KEY_PASSWORDS);
    if (s) return JSON.parse(s);
    localStorage.setItem(STORAGE_KEY_PASSWORDS, JSON.stringify(DEFAULT_PASSWORDS));
    return DEFAULT_PASSWORDS;
};

const savePasswords = (passwords: Record<number, string>) => {
    localStorage.setItem(STORAGE_KEY_PASSWORDS, JSON.stringify(passwords));
};

const loadDevices = (): Device[] => {
    const s = localStorage.getItem(STORAGE_KEY_DEVICES);
    if (s) return JSON.parse(s);
    localStorage.setItem(STORAGE_KEY_DEVICES, JSON.stringify(DEFAULT_DEVICES));
    return DEFAULT_DEVICES;
};

const saveDevices = (devices: Device[]) => {
    localStorage.setItem(STORAGE_KEY_DEVICES, JSON.stringify(devices));
};

const getMockUserFromToken = (): User | undefined => {
    const token = localStorage.getItem(MOCK_ACCESS_TOKEN_KEY);
    if (!token) return undefined;
    const userId = parseInt(token.split('_')[1]); // simple token format: "mock_ID"
    return loadUsers().find(u => u.id === userId);
};

export const api: ApiService = {
    // Auth
    async login(data: LoginRequest): Promise<LoginResponse> {
        await new Promise(r => setTimeout(r, 600)); // Simulate network latency
        
        const users = loadUsers();
        const passwords = loadPasswords();
        
        const user = users.find(u => u.username === data.username);
        
        let valid = false;
        if (user) {
            if (user.is_disabled) {
                // Return 403 error structure
                throw { response: { data: { error: "Account disabled" } } };
            }

            if (passwords[user.id] === data.password) {
                valid = true;
            }
        }

        if (valid && user) {
            const token = `mock_${user.id}_${Date.now()}`;
            localStorage.setItem(MOCK_ACCESS_TOKEN_KEY, token); // Store for getMe
            
            // Update last login
            user.last_login_at = new Date().toISOString();
            saveUsers(users);

            return {
                message: "Login successful",
                user: user,
                access_token: token,
                refresh_token: `mock_refresh_${user.id}`
            };
        }
        
        throw { response: { data: { error: "Invalid credentials" } } };
    },

    async logout(): Promise<void> {
        localStorage.removeItem(MOCK_ACCESS_TOKEN_KEY);
    },

    async getMe(): Promise<User> {
        await new Promise(r => setTimeout(r, 200));
        const user = getMockUserFromToken();
        if (user) {
            if (user.is_disabled) throw { response: { status: 401 } };
            return user;
        }
        throw { response: { status: 401 } };
    },

    // Users
    async getUsers(): Promise<User[]> {
        await new Promise(r => setTimeout(r, 400));
        return loadUsers();
    },

    async createUser(data: CreateUserRequest): Promise<{ user: User; password: string }> {
        await new Promise(r => setTimeout(r, 500));
        const users = loadUsers();
        if (users.find(u => u.username === data.username)) {
            throw new Error("Username taken");
        }
        
        const newUser: User = {
            id: Date.now(),
            username: data.username,
            role: 'user',
            force_password_change: true,
            is_disabled: false,
        };
        
        users.push(newUser);
        saveUsers(users);

        const passwords = loadPasswords();
        passwords[newUser.id] = "password";
        savePasswords(passwords);
        
        return { user: newUser, password: "password" };
    },

    async deleteUser(id: number): Promise<void> {
        await new Promise(r => setTimeout(r, 400));
        const currentUser = getMockUserFromToken();
        if (currentUser?.id === id) throw new Error("Cannot delete yourself");

        const users = loadUsers().filter(u => u.id !== id);
        saveUsers(users);
        
        const passwords = loadPasswords();
        delete passwords[id];
        savePasswords(passwords);
    },

    async updateUserRole(id: number, role: 'admin' | 'user'): Promise<void> {
        await new Promise(r => setTimeout(r, 300));
        const currentUser = getMockUserFromToken();
        if (currentUser?.id === id) throw new Error("Cannot change your own role");

        const users = loadUsers();
        const user = users.find(u => u.id === id);
        if (user) {
            user.role = role;
            saveUsers(users);
        }
    },

    async updateUserStatus(id: number, is_disabled: boolean): Promise<void> {
        await new Promise(r => setTimeout(r, 300));
        const currentUser = getMockUserFromToken();
        if (currentUser?.id === id && is_disabled) throw new Error("Cannot disable yourself");

        const users = loadUsers();
        const user = users.find(u => u.id === id);
        if (user) {
            user.is_disabled = is_disabled;
            saveUsers(users);
        }
    },

    async resetUserPassword(id: number, data: AdminResetPasswordRequest): Promise<AdminResetPasswordResponse> {
        await new Promise(r => setTimeout(r, 500));
        const users = loadUsers();
        const user = users.find(u => u.id === id);
        if (user) {
            user.force_password_change = true;
            saveUsers(users);
            
            const passwords = loadPasswords();
            const newPassword = data.new_password || `mock-${Date.now().toString(36)}`;
            passwords[id] = newPassword;
            savePasswords(passwords);

            return {
                message: "Password reset successfully",
                password: data.new_password ? undefined : newPassword
            };
        }
        throw new Error("User not found");
    },

    async changePassword(data: { old_password: string, new_password: string }): Promise<void> {
        await new Promise(r => setTimeout(r, 500));
        const users = loadUsers();
        const currentUser = getMockUserFromToken();
        if (!currentUser) throw new Error("Not logged in");

        const passwords = loadPasswords();
        if (passwords[currentUser.id] !== data.old_password) {
            throw { response: { data: { error: "Invalid current password" } } };
        }

        const user = users.find(u => u.id === currentUser.id);
        if (user) {
            user.force_password_change = false;
            saveUsers(users);
            
            passwords[currentUser.id] = data.new_password;
            savePasswords(passwords);
        }
    },

    // Devices
    async getDevices(): Promise<Device[]> {
        await new Promise(r => setTimeout(r, 300));
        return loadDevices();
    },

    async createDevice(data: CreateDeviceRequest): Promise<Device> {
        await new Promise(r => setTimeout(r, 500));
        const devices = loadDevices();
        const newDevice: Device = {
            id: Date.now(),
            name: data.name,
            mac_address: data.mac_address,
            ip_address: data.ip_address,
            broadcast_addr: data.broadcast_addr,
            icon: data.icon,
            is_online: false,
            can_shutdown: false, // Default for demo
        };
        devices.push(newDevice);
        saveDevices(devices);
        return newDevice;
    },

    async updateDevice(id: number, data: UpdateDeviceRequest): Promise<Device> {
        await new Promise(r => setTimeout(r, 500));
        const devices = loadDevices();
        const index = devices.findIndex(d => d.id === id);
        if (index !== -1) {
            devices[index] = { ...devices[index], ...data };
            saveDevices(devices);
            return devices[index];
        }
        throw new Error("Device not found");
    },

    async deleteDevice(id: number): Promise<void> {
        await new Promise(r => setTimeout(r, 400));
        const devices = loadDevices().filter(d => d.id !== id);
        saveDevices(devices);
    },

    async wakeDevice(id: number): Promise<void> {
        await new Promise(r => setTimeout(r, 800)); // Simulate magic packet sending
        // In demo, let's simulate it coming online after a short delay?
        // Or just say "Signal Sent".
        // Let's toggle it online for fun after 2 seconds (simulated background ping).
        setTimeout(() => {
            const devices = loadDevices();
            const device = devices.find(d => d.id === id);
            if (device) {
                device.is_online = true;
                device.last_seen_at = new Date().toISOString();
                saveDevices(devices);
            }
        }, 2000);
    },

    async shutdownDevice(id: number): Promise<void> {
        await new Promise(r => setTimeout(r, 800));
        setTimeout(() => {
            const devices = loadDevices();
            const device = devices.find(d => d.id === id);
            if (device) {
                device.is_online = false;
                saveDevices(devices);
            }
        }, 2000);
    }
};
