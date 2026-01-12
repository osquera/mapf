// API client for backend server verification
import type { Coordinate, GridMap, Solution, ValidationError } from '../types';

export interface VerifyRequest {
	wasmBytes: Uint8Array;
	map: {
		width: number;
		height: number;
		tiles: number[];
	};
	starts: Coordinate[];
	goals: Coordinate[];
}

export interface VerifyResponse {
	valid: boolean;
	solution: Solution | null;
	validation_errors: ValidationError[];
	stats: {
		instruction_count: number | null;
		execution_time_ms: number;
		cost: number | null;
		makespan: number | null;
	};
	error: string | null;
}

export interface SubmitRequest {
	solver_name: string;
	map_name: string;
	scenario_id: string;
	wasmBytes: Uint8Array;
	map: {
		width: number;
		height: number;
		tiles: number[];
	};
	starts: Coordinate[];
	goals: Coordinate[];
}

export interface SubmitResponse {
	submission_id: string;
	verification_id: string;
	message: string;
}

export interface LeaderboardEntry {
	username: string;
	solver_name: string;
	map_name: string;
	scenario_id: string;
	num_agents: number;
	cost: number | null;
	makespan: number | null;
	instruction_count: number | null;
	execution_time_ms: number;
	verified_at: string;
}

export interface RegisterRequest {
	username: string;
	email: string;
	key_name: string;
}

export interface RegisterResponse {
	user_id: string;
	api_key: string;
	message: string;
}

export class BackendClient {
	private baseUrl: string;
	private apiKey: string | null;

	constructor(baseUrl: string = 'http://localhost:3000', apiKey: string | null = null) {
		this.baseUrl = baseUrl;
		this.apiKey = apiKey;
	}

	setApiKey(apiKey: string) {
		this.apiKey = apiKey;
	}

	/**
	 * Verify a WASM solver on the backend (no authentication required)
	 */
	async verify(request: VerifyRequest): Promise<VerifyResponse> {
		const response = await fetch(`${this.baseUrl}/api/verify`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				wasmBytes: Array.from(request.wasmBytes),
				map: request.map,
				starts: request.starts,
				goals: request.goals
			})
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || 'Verification failed');
		}

		return response.json();
	}

	/**
	 * Submit a verified solver to the leaderboard (requires authentication)
	 */
	async submit(request: SubmitRequest): Promise<SubmitResponse> {
		if (!this.apiKey) {
			throw new Error('API key required for submissions');
		}

		const response = await fetch(`${this.baseUrl}/api/submit`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${this.apiKey}`
			},
			body: JSON.stringify({
				solver_name: request.solver_name,
				map_name: request.map_name,
				scenario_id: request.scenario_id,
				wasmBytes: Array.from(request.wasmBytes),
				map: request.map,
				starts: request.starts,
				goals: request.goals
			})
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || 'Submission failed');
		}

		return response.json();
	}

	/**
	 * Get leaderboard entries
	 */
	async getLeaderboard(mapName?: string, limit: number = 100): Promise<LeaderboardEntry[]> {
		const params = new URLSearchParams();
		if (mapName) params.set('map_name', mapName);
		params.set('limit', limit.toString());

		const response = await fetch(`${this.baseUrl}/api/leaderboard?${params}`);

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || 'Failed to fetch leaderboard');
		}

		const data = await response.json();
		return data.entries;
	}

	/**
	 * Register a new user and get an API key
	 */
	async register(request: RegisterRequest): Promise<RegisterResponse> {
		const response = await fetch(`${this.baseUrl}/api/auth/register`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(request)
		});

		if (!response.ok) {
			const error = await response.json();
			throw new Error(error.error || 'Registration failed');
		}

		return response.json();
	}

	/**
	 * Check if backend is available
	 */
	async healthCheck(): Promise<boolean> {
		try {
			const response = await fetch(`${this.baseUrl}/health`, {
				method: 'GET',
				signal: AbortSignal.timeout(5000)
			});
			return response.ok;
		} catch {
			return false;
		}
	}
}

// Singleton instance with environment-based URL
export const backendClient = new BackendClient(
	import.meta.env.VITE_BACKEND_URL || 'http://localhost:3000'
);
