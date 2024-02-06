import { db } from "$src/lib/db"
import { get, writable } from "svelte/store"



export type Project = {
    version: number,
    id: string,
    name: string,
    description: string,
    createdAt: number,
    updatedAt: number
    content: string
}




export function createProject(): Project {
    return {
        version: 1,
        id: "",
        name: "Unnamed",
        description: "",
        createdAt: new Date().getTime(),
        updatedAt: new Date().getTime(),
        content: 
`min x
s.t.
    /* write the constraints here */
    x >= y
where
    /* write the constants here */
    y = 10
define
    /* define the model's variables here */
    x as Real`
    }
}

type ProjectStore = {
    initialized: boolean,
    projects: Project[],
}

export function createProjectStore() {
    const { subscribe, update } = writable<ProjectStore>({
        initialized: false,
        projects: []
    })

    async function ensureInit() {
        const isInit = get({ subscribe }).initialized
        if (!isInit) {
            await syncProjectsWithStore()
            update(store => {
                store.initialized = true
                return store
            })
        }
    }
    async function createNewProject(name: string, description: string): Promise<Project> {
        await ensureInit()
        const project = createProject()
        project.name = name || "Unnamed"
        project.description = description
        const pr = await db.saveProject(project)
        update(store => {
            store.projects.push(pr)
            return store
        })
        return pr
    }
    async function updateProject(id: string, fields: Partial<Project>): Promise<Project> {
        await ensureInit()
        const project = await getProject(id)
        const toUpdate = { ...project, ...fields }
        delete toUpdate.id
        const pr = await db.updateProject(id, toUpdate)
        
        update(store => {
            const index = store.projects.findIndex(p => p.id === pr.id)
            if (index === -1) {
                throw new Error("Project not found")
            }
            store.projects[index] = pr
            return store
        })
        return pr
    }
    async function deleteProject(id: string) {
        await ensureInit()
        await db.deleteProject(id)
        update(store => {
            store.projects = store.projects.filter(p => p.id !== id)
            return store
        })
    }

    async function syncProjectsWithStore() {
        const projects = await db.loadProjects()
        update(store => {
            store.projects = projects
            return store
        })
    }
    async function getProject(id: string): Promise<Project | undefined> {
        await ensureInit()
        return get({ subscribe }).projects.find(p => p.id === id)
    }
    return {
        subscribe,
        createNewProject,
        updateProject,
        syncProjectsWithStore,
        deleteProject,
        getProject
    }
}

export const projectStore = createProjectStore()



