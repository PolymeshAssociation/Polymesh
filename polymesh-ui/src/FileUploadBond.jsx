import React from 'react';
import {Button, Label} from 'semantic-ui-react';
import {Bond} from 'oo7';
import {ReactiveComponent} from 'oo7-react';
import * as uuid from 'uuid';

export class FileUploadBond extends ReactiveComponent {
	constructor () {
		super(['content', 'disabled']);

		this.changed = this.changed.bind(this)
		this.state = { length: null }
		this.id = uuid.v1()
	}

	changed () {
		const fileButton = document.getElementById(this.id)
		const file = fileButton ? fileButton.files[0] : null
		
		if (file) {
			var fileReader = new FileReader()
			fileReader.onloadend = e => {
				let fileContents = new Uint8Array(e.target.result)
				this.props.bond.trigger(fileContents)
				this.setState({length: fileContents.length})
			}
			fileReader.readAsArrayBuffer(file)
		}
	}

	render () {
		return <div>
			<Button
				content={this.state.content}
				disabled={this.state.disabled}
				as="label"
				htmlFor={this.id}
				label={this.state.length
					? `${this.state.length} bytes`
					: null
				}
			></Button>
			<input
				hidden
				id={this.id}
				multiple
				type="file"
				onChange={this.changed}
			/>
		</div>
	}
}
