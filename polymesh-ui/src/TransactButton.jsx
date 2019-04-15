const React = require('react');
const {Bond} = require('oo7');
const {ReactiveComponent} = require('oo7-react');
const {post} = require('oo7-substrate');
const {Button} = require('semantic-ui-react');
const {TransactionProgressLabel, styleStatus} = require('./TransactionProgressLabel');

class TransactButton extends ReactiveComponent {
	constructor () {
		super(['content', 'disabled', 'enabled', 'positive', 'negative', 'active']);
		this.state = { index: 0, status: null };
		this.handleClick = this.handleClick.bind(this);
	}
	handleClick () {
		let begin = false;
		let s = this.state;
		if (s.status) {
			s.status = null;
		} else {
			s.index = 0;
			begin = true;
		}
		this.setState(s);

		if (begin) {
			this.execNext();
		}
	}
	execNext () {
		let s = this.state;
		let single = typeof(this.props.tx) === 'function' || this.props.tx.length === undefined;
		if ((single && s.index === 0) || s.index < this.props.tx.length) {
			let t = single ? this.props.tx : this.props.tx[s.index];
			s.status = typeof(t) === 'function'
				? t()
				: post(t);
			s.status.tie((x, i) => {
				if (this.props.order ? this.props.causal ? x.confirmed || x.scheduled : x.signed : x.requested) {
					this.execNext();
					s.status.untie(i);
				} else if (this.props.failed) {
					s.status.untie(i);
				}
			});
		}
		s.index++
		this.setState(s);
	}
	render () {
		if (!this.props.tx) {
			return (<span/>);
		}
		return <TransactButtonAux
			icon={this.props.icon}
			size={this.props.size}
			positive={this.state.positive}
			negative={this.state.negative}
			floated={this.props.floated}
			compact={this.props.compact}
			circular={this.props.circular}
			basic={this.props.basic}
			attached={this.props.attached}
			active={this.state.active}
			fluid={this.props.fluid}
			primary={this.props.primary}
			secondary={this.props.secondary}
			content={this.state.content}
			color={this.props.color}
			status={this.state.status}
			progress={{current: this.state.index, total: this.props.tx.length}}
			onClick={this.handleClick}
			statusText={this.props.statusText}
			statusIcon={this.props.statusIcon}
			colorPolicy={this.props.colorPolicy}
			disabled={Bond.all([this.props.tx]).ready().map(txReady => !txReady || this.state.disabled || !this.state.enabled)}
		/>
	}
}
TransactButton.defaultProps = {
	statusText: false,
	statusIcon: true,
	colorPolicy: 'button',
	enabled: true,
	order: true,
	causal: true
};

class TransactButtonAux extends ReactiveComponent {
	constructor() {
		super(['status', 'disabled']);
	}
	render() {
		let specialColor = this.props.primary || this.props.secondary;
		let done = this.state.status && (this.state.status.confirmed || this.state.status.scheduled || this.state.status.failed);
		let clickable = !this.state.status || done;
		let status = this.state.status && styleStatus(this.state.status);
		let statusColor = status ? status.color : null;
		let labelColor = (this.props.colorPolicy === 'button' && !specialColor ? this.props.color : null) || statusColor || this.props.color;
		let buttonColor = (this.props.colorPolicy === 'status' ? statusColor : this.props.color) || this.props.color || statusColor;
		return (<Button
			icon={this.state.status === null || !clickable ? this.props.icon : 'check'}
			size={this.props.size}
			positive={this.props.positive}
			negative={this.props.negative}
			floated={this.props.floated}
			compact={this.props.compact}
			circular={this.props.circular}
			basic={this.props.basic}
			attached={this.props.attached}
			active={this.props.active}
			fluid={this.props.fluid}
			primary={this.props.primary}
			secondary={this.props.secondary}
			content={done ? 'OK' : this.props.content}
			color={buttonColor}
			onClick={this.props.onClick}
			label={this.state.status ? (<TransactionProgressLabel
				value={this.state.status}
				current={this.props.progress.current}
				total={this.props.progress.total}
				showContent={this.props.statusText}
				showIcon={this.props.statusIcon}
				color={labelColor}
			/>) : null}
			disabled={!done && this.state.disabled}
		/>);
	}
}

module.exports = { TransactButton };
