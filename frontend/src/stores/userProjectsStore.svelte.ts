import {db} from "$src/lib/db"
import type {Pipes} from "@specy/rooc";
import {defaultPipe} from "$lib/pipePresets";


export type ProjectPipe = {
    pipe: Pipes
    open: boolean
}

export type Project = {
    version: number,
    id: string,
    name: string,
    description: string,
    createdAt: number,
    updatedAt: number
    content: string
    runtime: string
    runtimeVisible: boolean
    pipes: ProjectPipe[]
}

export function validateProject(project: Project): Project {
    return {...createProject(), ...project}
}
const defaultTs = `
/*
register({
    name: 'sqrt',
    description: 'Calculate the square root of a number',
    parameters: [['of_num', Primitive.Number]],
    returns: Primitive.Number,
    call: (num) => {
        return {type: "Number", value: Math.sqrt(num.value)}
    }
})
*/
`.trim()
export function createProject(): Project {
    return {
        version: 1,
        id: "",
        runtime: defaultTs,
        runtimeVisible: false,
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
    // write the constants here
    let y = 10
define
    // define the model's variables here
    x as PositiveReal`
        ,
        pipes: [...defaultPipe].map((p, i, arr) => ({pipe: p, open: i === (arr.length - 1)}))
    }
}



export function createProjectStore() {
    let initialized = $state(false)
    let projects = $state<Project[]>([])

    async function ensureInit() {
        if (!initialized) {
            await syncProjectsWithStore()
        }
    }

    async function createNewProject(name: string, description: string): Promise<Project> {
        await ensureInit()
        const project = createProject()
        project.name = name || "Unnamed"
        project.description = description
        const pr = await db.saveProject(project)
        projects.unshift(pr)
        return pr
    }

    async function updateProject(id: string, fields: Partial<Project>): Promise<Project> {
        await ensureInit()

        const project = await getProject(id)
        const toUpdate = {...project, ...fields}
        delete toUpdate.id
        const pr = await db.updateProject(id, toUpdate)
        const index = projects.findIndex(p => p.id === pr.id)
        if (index === -1) {
            throw new Error("Project not found")
        }
        projects[index] = pr
        return pr
    }

    async function deleteProject(id: string) {
        await ensureInit()
        await db.deleteProject(id)
        projects = projects.filter(p => p.id !== id)
    }

    async function syncProjectsWithStore() {
        const promise = await db.loadProjects()
        projects = promise.sort((a, b) => b.updatedAt - a.updatedAt)
            .map(validateProject)
        initialized = true
    }

    async function getProject(id: string): Promise<Project | undefined> {
        await ensureInit()
        return projects.find(p => p.id === id)
    }

    return {
        get projects(){ return projects},
        get initialized(){ return initialized},
        createNewProject,
        updateProject,
        syncProjectsWithStore,
        deleteProject,
        getProject
    }
}

export const projectStore = createProjectStore()



